use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;



#[proc_macro_derive(DiceBoard)]
pub fn derive_dice_board(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Get the size from the array type
    let size = if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            if let Some(field) = fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().map_or(false, |ident| ident == "cols"))
            {
                if let syn::Type::Array(array) = &field.ty {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(lit_int),
                        ..
                    }) = &array.len
                    {
                        lit_int.base10_parse::<usize>().unwrap()
                    } else {
                        panic!("Array size must be a literal")
                    }
                } else {
                    panic!("cols must be an array")
                }
            } else {
                panic!("Struct must have a cols field")
            }
        } else {
            panic!("Struct must have named fields")
        }
    } else {
        panic!("DiceBoard can only be derived for structs")
    };

    // Generate the implementation
    let expanded = quote! {
        impl DiceBoard for #name {
            fn new() -> Self {
                Self {
                    cols: [None; #size],
                }
            }

            fn insert_to(&mut self, col: usize, dice: Dice) -> Result<(), DiceError> {
                if col >= #size {
                    return Err(DiceError::InvalidColumn);
                }
                if self.cols[col].is_some() {
                    return Err(DiceError::ColumnOccupied);
                }
                self.cols[col] = Some(dice);
                Ok(())
            }

            fn strike_multi_to(&mut self, col: usize, dice_number: usize) -> Vec<Dice> {
                let mut struck = Vec::new();
                if col >= #size {
                    return struck;
                }
                for i in col..(#size.min(col + dice_number)) {
                    if let Some(dice) = self.cols[i].take() {
                        struck.push(dice);
                    }
                }
                struck
            }

            fn is_full(&self) -> bool {
                self.cols.iter().all(|x| x.is_some())
            }

            fn score(&self) -> u32 {
                self.cols
                    .iter()
                    .filter_map(|x| x.as_ref())
                    .map(|d| d.value as u32)
                    .sum()
            }
        }
    };

    TokenStream::from(expanded)
}
