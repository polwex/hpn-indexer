use crate::hyperware::process::hpn::{HpnMessage, Request as HpnRequest, Response as HpnResponse, SendRequest};
use crate::hyperware::process::tester::{Request as TesterRequest, Response as TesterResponse, RunRequest, FailResponse};

use hyperware_process_lib::{await_message, call_init, print_to_terminal, println, Address, ProcessId, Request, Response};

mod tester_lib;

wit_bindgen::generate!({
    path: "target/wit",
    world: "hpn-test-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [PartialEq, serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

fn handle_message (our: &Address) -> anyhow::Result<()> {
    let message = await_message().unwrap();

    if !message.is_request() {
        unimplemented!();
    }
    let source = message.source();
    if our.node != source.node {
        return Err(anyhow::anyhow!(
            "rejecting foreign Message from {:?}",
            source,
        ));
    }
    let TesterRequest::Run(RunRequest {
        input_node_names: node_names,
        ..
    }) = message.body().try_into()?;
    print_to_terminal(0, "hpn_test: a");
    assert!(node_names.len() >= 2);
    if our.node != node_names[0] {
        // we are not master node: return
        Response::new()
            .body(TesterResponse::Run(Ok(())))
            .send()
            .unwrap();
        return Ok(());
    }

    // we are master node

    let our_hpn_address = Address {
        node: our.node.clone(),
        process: ProcessId::new(Some("hpn"), "hpn", "template.os"),
    };
    let their_hpn_address = Address {
        node: node_names[1].clone(),
        process: ProcessId::new(Some("hpn"), "hpn", "template.os"),
    };

    // Send
    print_to_terminal(0, "hpn_test: b");
    let message: String = "hello".into();
    let _ = Request::new()
        .target(our_hpn_address.clone())
        .body(HpnRequest::Send(SendRequest {
            target: node_names[1].clone(),
            message: message.clone(),
        }))
        .send_and_await_response(15)?.unwrap();

    // Get history from receiver & test
    print_to_terminal(0, "hpn_test: c");
    let response = Request::new()
        .target(their_hpn_address.clone())
        .body(HpnRequest::History(our.node.clone()))
        .send_and_await_response(15)?.unwrap();
    if response.is_request() { fail!("hpn_test"); };
    let HpnResponse::History(messages) = response.body().try_into()? else {
        fail!("hpn_test");
    };
    let expected_messages = vec![HpnMessage {
        author: our.node.clone(),
        content: message,
    }];

    if messages != expected_messages {
        println!("{messages:?} != {expected_messages:?}");
        fail!("hpn_test");
    }

    Response::new()
        .body(TesterResponse::Run(Ok(())))
        .send()
        .unwrap();

    Ok(())
}

call_init!(init);
fn init(our: Address) {
    print_to_terminal(0, "begin");

    loop {
        match handle_message(&our) {
            Ok(()) => {},
            Err(e) => {
                print_to_terminal(0, format!("hpn_test: error: {e:?}").as_str());

                fail!("hpn_test");
            },
        };
    }
}
