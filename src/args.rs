use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;

/// Represents data that is either `T` or a `TokenStream`
pub enum TOrTokens<T> {
    T(T),
    Tokens(TokenStream),
}

/// Parses a TokenStream into a `T` where `T: FromMeta`
pub fn parse_macro_args<T>(input: TokenStream) -> TOrTokens<T>
where
    T: FromMeta,
{
    let args_list = match NestedMeta::parse_meta_list(input.into()) {
        Ok(v) => v,
        Err(e) => {
            return TOrTokens::Tokens(TokenStream::from(darling::Error::from(e).write_errors()));
        }
    };
    match T::from_list(&args_list) {
        Ok(v) => return TOrTokens::T(v),
        Err(e) => {
            return TOrTokens::Tokens(TokenStream::from(e.write_errors()));
        }
    };
}
