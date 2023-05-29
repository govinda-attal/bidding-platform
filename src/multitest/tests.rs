use cosmwasm_std::{coin, coins, Addr, Decimal};
use cw_multi_test::App;

use crate::{error::ContractError, msg::HighestBidResponse};

use super::BiddingContract;

const ATOM: &str = "atom";
const BIDDING_CONTRACT: &str = "bidding contract";
const ANTIQUE_ITEM: &str = "antique item";
#[test]
fn bid_example_flow() {
    let owner = Addr::unchecked("owner");
    let alex = Addr::unchecked("alex");
    let ann = Addr::unchecked("ann");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &alex, coins(25, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &ann, coins(25, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        BIDDING_CONTRACT,
        ANTIQUE_ITEM,
        ATOM,
        None,
        0,
        Decimal::percent(0),
    )
    .unwrap();

    let highest_bid = contract.query_highest_bid(&app).unwrap();
    assert_eq!(highest_bid, HighestBidResponse::default());
    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(0, ATOM)
    );

    contract.bid(&mut app, &alex, coin(15, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&alex, coin(15, ATOM))
    );

    contract.bid(&mut app, &ann, coin(17, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&ann, coin(17, ATOM))
    );

    contract.bid(&mut app, &ann, coin(2, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&ann, coin(19, ATOM))
    );

    let err = contract.bid(&mut app, &alex, coin(1, ATOM)).unwrap_err();
    assert_eq!(
        err,
        ContractError::BidRejected {
            highest_bid: coin(19, ATOM)
        }
    );

    contract.bid(&mut app, &alex, coin(5, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&alex, coin(20, ATOM))
    );

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(39, ATOM)
    );

    contract.close(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_balance(&owner, ATOM).unwrap(),
        coin(20, ATOM)
    );

    assert_eq!(
        app.wrap().query_balance(&alex, ATOM).unwrap(),
        coin(5, ATOM)
    );

    assert_eq!(app.wrap().query_balance(&ann, ATOM).unwrap(), coin(6, ATOM));

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(19, ATOM)
    );

    contract.retract(&mut app, &ann, None).unwrap();
    assert_eq!(
        app.wrap().query_balance(&ann, ATOM).unwrap(),
        coin(25, ATOM)
    );

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(0, ATOM)
    );
}

#[test]
fn bid_with_commission_flow() {
    let owner = Addr::unchecked("owner");
    let alex = Addr::unchecked("alex");
    let ann = Addr::unchecked("ann");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &alex, coins(25, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &ann, coins(25, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        BIDDING_CONTRACT,
        ANTIQUE_ITEM,
        ATOM,
        None,
        1,
        Decimal::percent(0),
    )
    .unwrap();

    let highest_bid = contract.query_highest_bid(&app).unwrap();
    assert_eq!(highest_bid, HighestBidResponse::default());
    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(0, ATOM)
    );

    contract.bid(&mut app, &alex, coin(15, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&alex, coin(14, ATOM))
    );
    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(14, ATOM)
    );
    assert_eq!(
        app.wrap().query_balance(&owner, ATOM).unwrap(),
        coin(1, ATOM)
    );

    contract.bid(&mut app, &ann, coin(17, ATOM)).unwrap();
    assert_eq!(
        contract.query_highest_bid(&app).unwrap(),
        HighestBidResponse::default().with_bidder(&ann, coin(16, ATOM))
    );

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(30, ATOM)
    );
    assert_eq!(
        app.wrap().query_balance(&owner, ATOM).unwrap(),
        coin(2, ATOM)
    );

    contract.close(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_balance(&owner, ATOM).unwrap(),
        coin(18, ATOM)
    );

    assert_eq!(
        app.wrap().query_balance(&alex, ATOM).unwrap(),
        coin(10, ATOM)
    );

    assert_eq!(app.wrap().query_balance(&ann, ATOM).unwrap(), coin(8, ATOM));

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(14, ATOM)
    );

    contract.retract(&mut app, &alex, None).unwrap();
    assert_eq!(
        app.wrap().query_balance(&alex, ATOM).unwrap(),
        coin(24, ATOM)
    );

    assert_eq!(
        app.wrap().query_balance(contract.addr(), ATOM).unwrap(),
        coin(0, ATOM)
    );
}

#[test]
fn owner_cannot_bid() {
    let owner = Addr::unchecked("owner");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        BIDDING_CONTRACT,
        ANTIQUE_ITEM,
        ATOM,
        None,
        0,
        Decimal::percent(0),
    )
    .unwrap();

    let err = contract.bid(&mut app, &owner, coin(5, ATOM)).unwrap_err();
    assert_eq!(err, ContractError::OwnerCannotBid {});
}

#[test]
fn open_bid_cannot_retract() {
    let owner = Addr::unchecked("owner");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10, ATOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        BIDDING_CONTRACT,
        ANTIQUE_ITEM,
        ATOM,
        None,
        0,
        Decimal::percent(0),
    )
    .unwrap();

    let err = contract.retract(&mut app, &owner, None).unwrap_err();
    assert_eq!(err, ContractError::BidOpen {});
}
