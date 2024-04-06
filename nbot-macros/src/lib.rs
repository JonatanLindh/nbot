use deluxe::ParseMetaItem;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, ItemStruct};

// #[proc_macro_derive(Command, attributes(path))]
// pub fn derive_asset_loader(item: TokenStream) -> TokenStream {
//     let mut ast: DeriveInput = syn::parse(item).unwrap();
//     let ident = &ast.ident;

//     quote! {
//         #[derive(
//             twilight_interactions::command::CommandModel,
//             twilight_interactions::command::CreateCommand,
//         )]
//         impl #ident {

//         }
//     }
//     .into()
// }

#[derive(ParseMetaItem, Debug)]
struct BotCommandArgs {
    name: String,
    desc: String,
    #[deluxe(default = true)]
    register: bool,
}

/// name: Command name
///
/// desc: Command description
///
/// register: Whether the command will be registered, default true
#[proc_macro_attribute]
pub fn bot_command(meta: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let ident = item.ident.clone();

    let BotCommandArgs {
        name,
        desc,
        register,
    } = match deluxe::parse(meta) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let reg = register.then(|| {
        quote! {
            inventory::submit! {
                crate::commands::CommandRegistrar::<'static>::new::<#ident>()
            }
        }
    });

    quote! {
        #[derive(
            twilight_interactions::command::CommandModel,
            twilight_interactions::command::CreateCommand,
        )]
        #[command(name = #name, desc = #desc)]
        #item

        #reg
    }
    .into()
}

/// name: Command name
///
/// desc: Command description
///
/// register: Whether the command will be registered, default true
#[proc_macro_attribute]
pub fn super_command(meta: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);
    dbg!(&item);
    let ident = item.ident;
    let vis = item.vis;

    let BotCommandArgs {
        name,
        desc,
        register,
    } = match deluxe::parse(meta) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let reg = register.then(|| {
        quote! {
            inventory::submit! {
                crate::commands::CommandRegistrar::<'static>::new::<#ident>()
            }
        }
    });

    let (v, v_a): (Vec<_>, Vec<_>) = item
        .variants
        .clone()
        .into_iter()
        .map(|v| (v.ident, v.attrs))
        .unzip();

    quote! {
        #[derive(
            twilight_interactions::command::CommandModel,
            twilight_interactions::command::CreateCommand,
        )]
        #[command(name = #name, desc = #desc)]
        #vis enum #ident {
            #(
                #(#v_a)*
                #v(#v),
            )*
        }

        #reg

        #[async_trait::async_trait]
        impl crate::commands::Command for #ident {
            async fn _run(
                self,
                app: std::sync::Arc<App>,
                interaction: twilight_model::application::interaction::Interaction
            ) -> anyhow::Result<()> {
                use #ident::*;

                match self {#(
                    #v(command) => command._run(app, interaction),
                )*}
                .await
            }
        }
    }
    .into()
}
