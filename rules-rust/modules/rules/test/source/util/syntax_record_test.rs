use crate::source::model::DeclaredVisibility;
use crate::source::util::{find_declared_visibility, find_path_attribute};

#[test]
fn given_dynamic_and_nonstring_paths_when_read_then_neither_is_a_file_path() {
    for source in ["#[path=source::PATH] mod item;", "#[path=1] mod item;"] {
        let module = syn::parse_str::<syn::ItemMod>(source).expect("attribute syntax");
        assert_eq!(find_path_attribute(&module.attrs), None);
    }
    let visibility = syn::parse_str("pub(super)").expect("restricted visibility");
    assert_eq!(
        find_declared_visibility(&visibility),
        DeclaredVisibility::Restricted
    );
}
