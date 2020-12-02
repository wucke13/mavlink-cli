use skim::prelude::*;
use std::io;

fn options() -> SkimOptions<'static> {
    let options = SkimOptionsBuilder::default()
        .height(Some("95%"))
        .multi(true)
        //.exact(true)
        .color(Some("16"))
        //.tiebreak(Some(String::from("begin,-end")))
        .preview(Some("")) // preview should be specified to enable preview window
        .build()
        .unwrap();
    options
}

pub fn select<T>(parameters: &[T]) -> io::Result<Vec<T>>
where
    T: Clone + SkimItem,
{
    let options = options();
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    for param in parameters {
        let _ = tx_item.send(Arc::new(param.clone()));
    }

    drop(tx_item); // so that skim could know when to stop waiting for more items.

    Ok(Skim::run_with(&options, Some(rx_item))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "User hit CTRL+C"))?
        .selected_items
        .into_iter()
        .filter_map(|item| (*item).as_any().downcast_ref::<T>().cloned())
        .collect())
}
