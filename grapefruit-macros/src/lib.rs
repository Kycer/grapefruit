
#[macro_use]
mod util;
mod common;
mod table;


use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_derive(GrapefruitTable, attributes(table, id, column))]
#[proc_macro_error]
pub fn table(input: TokenStream) -> TokenStream {
    table::impl_table(input)
}
