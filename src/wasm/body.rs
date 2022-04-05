#[cfg(feature = "multipart")]
use super::multipart::Form;
/// dox
use bytes::Bytes;
use js_sys::Uint8Array;
use std::fmt;
use wasm_bindgen::JsValue;

#[cfg(feature = "stream")]
use wasm_streams::readable::ReadableStream;

/// The body of a `Request`.
///
/// In most cases, this is not needed directly, as the
/// [`RequestBuilder.body`][builder] method uses `Into<Body>`, which allows
/// passing many things (like a string or vector of bytes).
///
/// [builder]: ./struct.RequestBuilder.html#method.body
pub struct Body {
    inner: Inner,
}

enum Inner {
    Bytes(Bytes),
    /// MultipartForm holds a multipart/form-data body.
    #[cfg(feature = "multipart")]
    MultipartForm(Form),
    /// MultipartPart holds the body of a multipart/form-data part.
    #[cfg(feature = "multipart")]
    MultipartPart(Bytes),
    /// Streaming body contain a ReadableStream
    #[cfg(feature = "stream")]
    Streaming(ReadableStream),
}

impl Body {
    /// Returns a reference to the internal data of the `Body`.
    ///
    /// `None` is returned, if the underlying data is a multipart form.
    #[inline]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.inner {
            Inner::Bytes(bytes) => Some(bytes.as_ref()),
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(_) => None,
            #[cfg(feature = "multipart")]
            Inner::MultipartPart(bytes) => Some(bytes.as_ref()),
            #[cfg(feature = "stream")]
            Inner::Streaming(_) => None,
        }
    }
    pub(crate) fn to_js_value(&self) -> crate::Result<JsValue> {
        match &self.inner {
            Inner::Bytes(body_bytes) => {
                let body_bytes: &[u8] = body_bytes.as_ref();
                let body_uint8_array: Uint8Array = body_bytes.into();
                let js_value: &JsValue = body_uint8_array.as_ref();
                Ok(js_value.to_owned())
            }
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(form) => {
                let form_data = form.to_form_data()?;
                let js_value: &JsValue = form_data.as_ref();
                Ok(js_value.to_owned())
            }
            #[cfg(feature = "multipart")]
            Inner::MultipartPart(body_bytes) => {
                let body_bytes: &[u8] = body_bytes.as_ref();
                let body_uint8_array: Uint8Array = body_bytes.into();
                let body_array = js_sys::Array::new();
                body_array.push(&body_uint8_array);
                let js_value: &JsValue = body_array.as_ref();
                Ok(js_value.to_owned())
            }
            #[cfg(feature = "stream")]
            Inner::Streaming(readable_stream) => {
                let js_value: &JsValue = readable_stream.as_raw().as_ref();

                Ok(js_value.to_owned())
            }
        }
    }

    #[inline]
    #[cfg(feature = "multipart")]
    pub(crate) fn from_form(f: Form) -> Body {
        Self {
            inner: Inner::MultipartForm(f),
        }
    }

    /// into_part turns a regular body into the body of a mutlipart/form-data part.
    #[cfg(feature = "multipart")]
    pub(crate) fn into_part(self) -> Body {
        match self.inner {
            Inner::Bytes(bytes) => Self {
                inner: Inner::MultipartPart(bytes),
            },
            Inner::MultipartForm(form) => Self {
                inner: Inner::MultipartForm(form),
            },
            Inner::MultipartPart(bytes) => Self {
                inner: Inner::MultipartPart(bytes),
            },
            #[cfg(feature = "stream")]
            Inner::Streaming(readable_stream) => Self {
                inner: Inner::Streaming(readable_stream)
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match &self.inner {
            Inner::Bytes(bytes) => bytes.is_empty(),
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(form) => form.is_empty(),
            #[cfg(feature = "multipart")]
            Inner::MultipartPart(bytes) => bytes.is_empty(),
            #[cfg(feature = "stream")]
            Inner::Streaming(_) => false,
        }
    }

    pub(crate) fn try_clone(&self) -> Option<Body> {
        match &self.inner {
            Inner::Bytes(bytes) => Some(Self {
                inner: Inner::Bytes(bytes.clone()),
            }),
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(_) => None,
            #[cfg(feature = "multipart")]
            Inner::MultipartPart(bytes) => Some(Self {
                inner: Inner::MultipartPart(bytes.clone()),
            }),
            #[cfg(feature = "stream")]
            Inner::Streaming(_) => None,
        }
    }
}

#[cfg(feature = "stream")]
impl From<ReadableStream> for Body {
    #[inline]
    fn from(readable_stream: ReadableStream) -> Body {
        Body {
            inner: Inner::Streaming(readable_stream),
        }
    }
}

impl From<Bytes> for Body {
    #[inline]
    fn from(bytes: Bytes) -> Body {
        Body {
            inner: Inner::Bytes(bytes),
        }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(vec: Vec<u8>) -> Body {
        Body {
            inner: Inner::Bytes(vec.into()),
        }
    }
}

impl From<&'static [u8]> for Body {
    #[inline]
    fn from(s: &'static [u8]) -> Body {
        Body {
            inner: Inner::Bytes(Bytes::from_static(s)),
        }
    }
}

impl From<String> for Body {
    #[inline]
    fn from(s: String) -> Body {
        Body {
            inner: Inner::Bytes(s.into()),
        }
    }
}

impl From<&'static str> for Body {
    #[inline]
    fn from(s: &'static str) -> Body {
        s.as_bytes().into()
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::Body;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;

    
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: String);
    }

    #[wasm_bindgen_test]
    async fn test_body() {
        let body = Body::from("TEST");
        assert_eq!([84, 69, 83, 84], body.as_bytes().unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_body_js_static_str() {
        let body_value = "TEST";
        let body = Body::from(body_value);

        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(
            body.to_js_value()
                .expect("could not convert body to JsValue")
                .as_ref(),
        ));

        let js_req = web_sys::Request::new_with_str_and_init("", &init)
            .expect("could not create JS request");
        let text_promise = js_req.text().expect("could not get text promise");
        let text = crate::wasm::promise::<JsValue>(text_promise)
            .await
            .expect("could not get request body as text");

        assert_eq!(text.as_string().expect("text is not a string"), body_value);
    }
    #[wasm_bindgen_test]
    async fn test_body_js_string() {
        let body_value = "TEST".to_string();
        let body = Body::from(body_value.clone());

        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(
            body.to_js_value()
                .expect("could not convert body to JsValue")
                .as_ref(),
        ));

        let js_req = web_sys::Request::new_with_str_and_init("", &init)
            .expect("could not create JS request");
        let text_promise = js_req.text().expect("could not get text promise");
        let text = crate::wasm::promise::<JsValue>(text_promise)
            .await
            .expect("could not get request body as text");

        assert_eq!(text.as_string().expect("text is not a string"), body_value);
    }

    #[wasm_bindgen_test]
    async fn test_body_js_static_u8_slice() {
        let body_value: &'static [u8] = b"\x00\x42";
        let body = Body::from(body_value);

        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(
            body.to_js_value()
                .expect("could not convert body to JsValue")
                .as_ref(),
        ));

        let js_req = web_sys::Request::new_with_str_and_init("", &init)
            .expect("could not create JS request");

        let array_buffer_promise = js_req
            .array_buffer()
            .expect("could not get array_buffer promise");
        let array_buffer = crate::wasm::promise::<JsValue>(array_buffer_promise)
            .await
            .expect("could not get request body as array buffer");

        let v = Uint8Array::new(&array_buffer).to_vec();

        assert_eq!(v, body_value);
    }

    #[wasm_bindgen_test]
    async fn test_body_js_vec_u8() {
        let body_value = vec![0u8, 42];
        let body = Body::from(body_value.clone());

        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(
            body.to_js_value()
                .expect("could not convert body to JsValue")
                .as_ref(),
        ));

        let js_req = web_sys::Request::new_with_str_and_init("", &init)
            .expect("could not create JS request");

        let array_buffer_promise = js_req
            .array_buffer()
            .expect("could not get array_buffer promise");
        let array_buffer = crate::wasm::promise::<JsValue>(array_buffer_promise)
            .await
            .expect("could not get request body as array buffer");

        let v = Uint8Array::new(&array_buffer).to_vec();

        assert_eq!(v, body_value);
    }

    #[wasm_bindgen_test]
    async fn test_body_js_stream() {
        use futures_util::StreamExt;
        use wasm_bindgen::JsCast;
        use wasm_streams::readable::ReadableStream;
        use web_sys::Blob;
        use js_sys::Array;

        let in_data = b"Hello World!";

        let u8_array = Uint8Array::new_with_length(in_data.len() as u32);
        u8_array.copy_from(in_data);

        let array = Array::new();
        array.push(&u8_array);
        let blob = Blob::new_with_u8_array_sequence(&array).expect("Blob Construction");
    
        let stream = ReadableStream::from_raw(blob.stream().unchecked_into());

        let body = Body::from(stream);

        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(
            body.to_js_value()
                .expect("could not convert body to JsValue")
                .as_ref(),
        ));

        let js_req = web_sys::Request::new_with_str_and_init("", &init)
            .expect("could not create JS request");

        let body = js_req.body().expect("Browser support ReadableStream");
        let stream = ReadableStream::from_raw(body.unchecked_into());

        let out_stream = stream.into_stream().map(|buf_js| -> crate::Result<Vec<u8>>{
            let buffer = Uint8Array::new(
                &buf_js
                    .map_err(crate::error::wasm)
                    .map_err(crate::error::decode)?,
            );
            let mut bytes = vec![0; buffer.length() as usize];
            buffer.copy_to(&mut bytes);
            Ok(bytes)
        });

        let out_data: Vec<Vec<u8>> = out_stream.filter_map(|item| async move {
            match item {
                Ok(bytes) => Some(bytes),
                Err(_) => None,
            }
        }).collect().await;

        let out_data: Vec<u8> = out_data.into_iter().flatten().collect();

        assert_eq!(out_data, in_data);
    }
}
