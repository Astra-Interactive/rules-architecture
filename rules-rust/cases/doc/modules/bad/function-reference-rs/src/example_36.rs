pub(crate) struct UserId(pub(crate) u64);

pub(crate) struct ItemContent {
    pub(crate) label: String,
}

pub(crate) struct CartItem {
    pub(crate) name: String,
}

impl CartItem {
    pub(crate) fn to_item_content(&self) -> ItemContent {
        ItemContent {
            label: self.name.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct OrderError;

pub(crate) struct Order;

#[derive(Debug)]
pub(crate) enum PlaceOrderError {
    Order(OrderError),
}

pub(crate) fn parse_order(raw: &str) -> Result<Order, OrderError> {
    if raw.is_empty() {
        return Err(OrderError);
    }
    Ok(Order)
}

pub(crate) struct Summary {
    pub(crate) user_ids: Vec<UserId>,
    pub(crate) contents: Vec<ItemContent>,
    pub(crate) name_count: usize,
    pub(crate) labels: Vec<String>,
    pub(crate) order: Order,
}

pub(crate) fn build_summary(
    ids: Vec<u64>,
    items: &[CartItem],
    text: &str,
    codes: &[u32],
    raw: &str,
) -> Result<Summary, PlaceOrderError> {
    let user_ids: Vec<UserId> = ids.into_iter().map(|id| UserId(id)).collect();
    let contents: Vec<ItemContent> = items.iter().map(|item| item.to_item_content()).collect();
    let names: Vec<&str> = text.lines().map(|line| line.trim()).collect();
    let labels: Vec<String> = codes.iter().map(|code| code.to_string()).collect();
    let order = parse_order(raw).map_err(|error| PlaceOrderError::Order(error))?;
    Ok(Summary {
        user_ids,
        contents,
        name_count: names.len(),
        labels,
        order,
    })
}
