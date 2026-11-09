use crate::cart::model::{CartItem, ItemContent, Summary, UserId};
use crate::order::model::PlaceOrderError;
use crate::order::util::parse_order;

pub(crate) fn build_summary(
    ids: Vec<u64>,
    items: &[CartItem],
    text: &str,
    codes: &[u32],
    raw: &str,
) -> Result<Summary, PlaceOrderError> {
    let user_ids: Vec<UserId> = ids.into_iter().map(UserId).collect();
    let contents: Vec<ItemContent> = items.iter().map(CartItem::to_item_content).collect();
    let names: Vec<&str> = text.lines().map(str::trim).collect();
    let labels: Vec<String> = codes.iter().map(ToString::to_string).collect();
    let order = parse_order(raw).map_err(PlaceOrderError::Order)?;
    Ok(Summary {
        user_ids,
        contents,
        name_count: names.len(),
        labels,
        order,
    })
}
