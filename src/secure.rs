use actix_web::{http::StatusCode, HttpResponse};
use crypto::{
    hmac::Hmac, mac::{Mac, MacResult}, sha1::Sha1,
};

pub struct Forbidden;

impl Into<HttpResponse> for Forbidden {
    fn into(self) -> HttpResponse {
        HttpResponse::new(StatusCode::FORBIDDEN)
    }
}

pub fn verify(data: &[u8], key: &[u8], signature: &[u8]) -> Result<(), Forbidden> {
    let mut hmac = Hmac::new(Sha1::new(), key);
    hmac.input(data);

    // Constant time comparison inside.
    if hmac.result() == MacResult::new(signature) {
        Ok(())
    } else {
        Err(Forbidden)
    }
}
