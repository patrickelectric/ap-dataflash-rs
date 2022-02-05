use std::io::{Read, Write};

use serde::Deserialize;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote;

macro_rules! ident {
    ($a:expr) => {{
        quote::format_ident!("{}", $a)
    }};
}

fn emit_struct(log_format: &LogFormat) -> TokenStream {
    let comment = &log_format.description;
    let struct_name = quote::format_ident!("{}Struct", log_format.name.to_case(Case::Pascal));
    let variables: Vec<TokenStream> = log_format
        .fields
        .fields
        .iter()
        .map(|field| {
            let comment = field.description.clone();
            let comment = comment.trim();

            let name = if field.name.parse::<u8>().is_ok() {
                quote::format_ident!("flag_{}", &field.name)
            } else {
                quote::format_ident!("field_{}", &field.name.to_case(Case::Snake))
            };

            quote! {
                #[doc = #comment]
                pub #name: LogType,
            }
        })
        .collect();
    quote! {
        #[derive(Debug, Clone, PartialEq, Default)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[doc = #comment]
        pub struct #struct_name {
            #(#variables)*
        }

    }
}

pub fn generate<R: Read, W: Write>(input: &mut R, output_rust: &mut W) {
    let log_messages = parse_description(input);

    let messages_enum = log_messages
        .log_formats
        .iter()
        .map(|format| {
            let pascal_message_name = ident!(format.name.to_case(Case::Pascal));
            let pascal_struct_name = quote::format_ident!("{}Struct", pascal_message_name);
            quote!(#pascal_message_name(#pascal_struct_name),)
        })
        .collect::<Vec<TokenStream>>();

    let messages_structs = log_messages
        .log_formats
        .iter()
        .map(|format| emit_struct(format))
        .collect::<Vec<TokenStream>>();

    let code = quote! {
        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum LogType {
            String(String),
            Number(i64),
            Float(f64),
            StringList(Vec<String>),
        }

        impl Default for LogType {
            fn default() -> Self {
                LogType::Number(0)
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum Messages {
            #(#messages_enum)*
        }

        #(#messages_structs)*
    };

    writeln!(output_rust, "{}", code).unwrap();
}

#[derive(Debug, Deserialize)]
struct Field {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct Fields {
    #[serde(rename = "field", default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
struct LogFormat {
    pub name: String,
    pub description: String,
    pub fields: Fields,
}

#[derive(Debug, Deserialize)]
struct LogMessages {
    #[serde(rename = "logformat", default)]
    pub log_formats: Vec<LogFormat>,
}

fn parse_description(file: &mut dyn Read) -> LogMessages {
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .expect("Failed to read content");

    return quick_xml::de::from_str(&file_content).unwrap();
}
