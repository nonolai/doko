//! # doko
//!
//! `doko` provides the procedural macro [`doko::doko!`](doko!) to enable running specific methods
//! in known submodules without explicitly importing those modules.
//!
//! # Examples
//!
//! For more in-depth examples, see the README and `examples` directory in the repository.
//!
//! ```ignore
//! doko::doko!("examples/utilities", "run", () -> u32);
//!
//! fn main() {
//!     assert_eq!(1, doko_run("first")());
//! }
//! ```

use crate::error::{Error, Result};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, ParenthesizedGenericArguments, Token};

mod error;

/// Arguments to the proc_macro, after being fully parsed and structured.
struct DokoArgs {
    path: LitStr,
    method: Ident,
    signature: ParenthesizedGenericArguments,
}

impl Parse for DokoArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        input.parse::<Token![,]>()?;
        let method_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let signature = input.parse()?;

        Ok(DokoArgs {
            path,
            method: Ident::new(&format!("{}", method_name.value()), Span::call_site()),
            signature,
        })
    }
}

/// All data needed to properly include and call methods in a particular submodule.
struct SubmoduleData {
    ident: Ident,
    name: String,
    include: TokenStream2,
}

/// Provides a function that can call some shared method in our included submodules by the modules
/// name.
///
/// Usage of this macro starts by specifying the module whose submodules should be included, the
/// name of the function shared between those submodules, and a type signature for the function.
///
/// ```ignore
/// doko::doko!("src/utilities", "my_execute", (&str) -> u32);
/// ```
///
/// Behind the scene, this includes all submodules directly inside of that module (i.e.
/// `src/utilities/*.rs`), and constructs a function named `doko_<method_name>` that can be used to
/// call the method with a submodule's name.
///
/// ```ignore
/// let i: u32 = doko_my_execute("foo")("argument"); // Executes `utilities::foo::my_execute`
/// ```
#[proc_macro]
pub fn doko(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DokoArgs);
    match tokens_for_input(input, true) {
        Ok(tokens) => tokens,
        Err(err) => TokenStream::from(syn::Error::new(Span::call_site(), err).to_compile_error()),
    }
}

/// Provides a function that can call some shared method in our included submodules by the modules
/// name. Skips including the modules, to allow for calling multiple shared methods per module.
#[proc_macro]
pub fn doko_skip_mods(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DokoArgs);
    match tokens_for_input(input, false) {
        Ok(tokens) => tokens,
        Err(err) => TokenStream::from(syn::Error::new(Span::call_site(), err).to_compile_error()),
    }
}

/// Perform the heavy lifting for the macro. Does all the actual work whereas the [`doko!`] just
/// parses input and checks the `Result` returned by this function.
fn tokens_for_input(input: DokoArgs, include_mods: bool) -> Result<TokenStream> {
    let enclosing_modules = get_enclosing_modules(&input.path.value())?;
    let submod_data = get_submodule_data(&input.path.value())?;
    let outer_mod = build_module_includes(&submod_data, &enclosing_modules);

    let registry = build_registry(
        &submod_data,
        &enclosing_modules,
        &input.method,
        &input.signature,
    );

    if include_mods {
        Ok(TokenStream::from(quote! {
             #outer_mod
             #registry
        }))
    } else {
        Ok(TokenStream::from(quote! {
             #registry
        }))
    }
}

/// For the directory being included, returns a Vec containing the identifiers of all the modules
/// the directory's submodules are enclosed in.
///
/// For instance, if the directory being included is `src/foo/bar/baz/<module.rs files>`, this
/// function will return vec!['foo', 'bar', 'baz'].
///
/// Returns an Error if any of the path components can't be parsed into a valid UTF-8 string.
fn get_enclosing_modules<P: AsRef<Path>>(directory: &P) -> Result<Vec<Ident>> {
    directory
        .as_ref()
        .components()
        .skip(1) // There's always a root folder, i.e. src, tests, or examples
        .map(|section| {
            let section_name = section
                .as_os_str()
                .to_str()
                .ok_or(Error::Utf8(section.as_os_str().to_os_string()))?;
            Ok(Ident::new(section_name, Span::call_site()))
        })
        .collect()
}

/// For the directory being included, returns a Vec containing metadata of each module defined
/// within that directory.
///
/// For instance, if the directory being included is `src/foo/bar/` and it contains `a.rs`,
/// `b.rs`, and `c.rs`, this function will return metadata for the submodules `foo::bar::a`,
/// `foo::bar::b`, and `foo::bar::c`.
///
/// Returns an Error if any of the submodules' absolute paths can't be parsed into UTF-8.
fn get_submodule_data<P: AsRef<Path> + AsRef<OsStr>>(directory: &P) -> Result<Vec<SubmoduleData>> {
    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => PathBuf::from(manifest_dir).join(directory),
        None => PathBuf::from(directory),
    };

    Ok(source_file_names(dir)?
        .into_iter()
        .map(|name| data_for_submodule(name))
        .collect())
}

/// Gets all metadata needed to include and call a particular submodule. Performs normalization of
/// hyphens in the module's name, and constructs an updated `mod` TokenStream if such normalization
/// is needed.
fn data_for_submodule(name: String) -> SubmoduleData {
    if name.contains('-') {
        let path = format!("{}.rs", name);
        let name = name.replace('-', "_").to_string();
        let ident = Ident::new(&name.replace('-', "_"), Span::call_site());
        SubmoduleData {
            ident: ident.clone(),
            name,
            include: quote! {
                #[path = #path]
                pub mod #ident;
            },
        }
    } else {
        let ident = Ident::new(&name, Span::call_site());
        SubmoduleData {
            ident: ident.clone(),
            name: name.to_string(),
            include: quote! {
                pub mod #ident;
            },
        }
    }
}

/// Constructs a TokenStream including a list of submodules, additionally taking into account the
/// supermodules that they are included in, and the orders thereof.
fn build_module_includes(
    submodules: &Vec<SubmoduleData>,
    enclosing_modules: &Vec<Ident>,
) -> TokenStream2 {
    let inner_modules =
        TokenStream2::from_iter(submodules.iter().map(|submod| submod.include.clone()));
    enclosing_modules
        .iter()
        .rev()
        .fold(inner_modules, |stream, module| {
            TokenStream2::from(quote!( pub mod #module { #stream } ))
        })
}

/// Constructs a TokenStream for a function that can call some shared method in our included
/// submodules by the modules name (as an &str).
///
/// The output function's signature is `pub fn doko_<method>(module_name: &str) -> <return type>`.
fn build_registry(
    submodules: &Vec<SubmoduleData>,
    enclosing_modules: &Vec<Ident>,
    method: &Ident,
    signature: &ParenthesizedGenericArguments,
) -> TokenStream2 {
    let gen_method_name = format_ident!("doko_{}", method);
    let args = &signature.inputs;
    let return_type = &signature.output;
    let prefix = enclosing_modules
        .iter()
        .fold(quote! { crate }, |ident, next| quote! { #ident::#next });
    let calls = TokenStream2::from_iter(
        submodules
            .iter()
            .map(|submod| get_call_for_submodule(submod, &prefix, method)),
    );

    TokenStream2::from(quote!(
        pub fn #gen_method_name(module_name: &str) -> fn(#args) #return_type {
            match module_name {
                #calls
                unknown => panic!("unknown module: {}", unknown),
            }
        }
    ))
}

/// Builds the match arm for a particular submodule in our "registry" function by combining various
/// pieces of metadata about the submodule.
fn get_call_for_submodule(
    submod: &SubmoduleData,
    prefix: &TokenStream2,
    method: &Ident,
) -> TokenStream2 {
    let ident = &submod.ident;
    let key = LitStr::new(&submod.name, Span::call_site());
    quote! {
        #key => #prefix::#ident::#method,
    }
}

/// Gets all modules to be included. Taken from Automod.
fn source_file_names<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut failures = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
            continue;
        }

        let path = Path::new(&file_name);
        if path.extension() == Some(OsStr::new("rs")) {
            match file_name.into_string() {
                Ok(mut utf8) => {
                    utf8.truncate(utf8.len() - ".rs".len());
                    names.push(utf8);
                }
                Err(non_utf8) => {
                    failures.push(non_utf8);
                }
            }
        }
    }

    failures.sort();
    if let Some(failure) = failures.into_iter().next() {
        return Err(Error::Utf8(failure));
    }

    if names.is_empty() {
        return Err(Error::Empty);
    }

    names.sort();
    Ok(names)
}
