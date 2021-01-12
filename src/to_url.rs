use std::path::Path;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

pub trait ToUrl {
    fn to_url(&self) -> String;
}

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');
const USERINFO: &AsciiSet = &PATH
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

impl<P: AsRef<Path>> ToUrl for P {
    fn to_url(&self) -> String {
        let escaped_components: Vec<String> = self.as_ref()
            .iter()
            .map(|component| {
                let enc = utf8_percent_encode(component.to_str().unwrap(), USERINFO);
                enc.to_string()
            })
            .collect();
        escaped_components.join("/")
    }
}
/*
impl ToUrl for String {
    fn to_url(&self) -> String {
        utf8_percent_encode(self, USERINFO).to_string()
    }    
}
 */