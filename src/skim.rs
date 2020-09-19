use crate::parameters::definitions::Definition;
use crate::parameters::Parameter;
use skim::prelude::*;

fn options() -> SkimOptions<'static> {
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some("")) // preview should be specified to enable preview window
        .build()
        .unwrap();
    options
}

fn select_parameter(parameters: Vec<Parameter>) {
    let options = options();
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    for param in parameters {
        let _ = tx_item.send(Arc::new(param.clone()));
    }

    drop(tx_item); // so that skim could know when to stop waiting for more items.

    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.into_iter() {
        //let param = (*item).as_any_mut().downcast_mut::<Parameter>().unwrap();
        //param.mutate();
        print!("{}{}", item.output(), "\n");
    }
}

pub fn select_params(parameters: &[Parameter]) {
    let options = options();
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    for param in parameters {
        let _ = tx_item.send(Arc::new(param.clone()));
    }

    drop(tx_item); // so that skim could know when to stop waiting for more items.

    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.into_iter() {
        let _param = (*item).as_any().downcast_ref::<Parameter>().unwrap();
        //param.mutate();
        print!("{}{}", item.output(), "\n");
    }
}

pub fn select_definition(definitions: &[Definition]) {
    let options = options();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

    for def in definitions {
        let _ = tx_item.send(Arc::new(def.clone()));
    }

    drop(tx_item); // so that skim could know when to stop waiting for more items.

    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.into_iter() {
        let def = (*item).as_any().downcast_ref::<Definition>().unwrap();
        let width = std::cmp::min(textwrap::termwidth(), 110);

        println!("\n{}", def.description(width));
    }
}
