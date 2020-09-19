use std::io;
use std::pin::Pin;

use skim::prelude::*;
use smol::prelude::*;

fn options() -> SkimOptions<'static> {
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some("")) // preview should be specified to enable preview window
        .build()
        .unwrap();
    options
}

pub async fn select<T>(mut items: Pin<Box<dyn Stream<Item = T>>>) -> io::Result<Vec<T>>
where
    T: Clone + SkimItem,
{
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    futures::join!(
        async {
            while let Some(item) = (items.next()).await {
                let _ = tx_item.send(Arc::new(item));
            }
            drop(tx_item); // so that skim could know when to stop waiting for more items.
        },
        smol::spawn(async {
            let options = options();
            Ok(Skim::run_with(&options, Some(rx_item))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "User hit CTRL+C"))?
                .selected_items
                .into_iter()
                .filter_map(|item| (*item).as_any().downcast_ref::<T>().cloned())
                .collect())
        })
    )
    .1
}
