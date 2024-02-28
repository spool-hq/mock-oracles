use anchor_lang::prelude::*;
use bytemuck::{cast_slice_mut, from_bytes_mut, try_cast_slice_mut, Pod, PodCastError};
use pyth_sdk_solana::state::{
    AccountType, PriceAccount, PriceStatus, ProductAccount, MAGIC, PROD_ACCT_SIZE, PROD_ATTR_SIZE,
    VERSION_2,
};
use std::cell::RefMut;
use std::mem::size_of;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const QUOTE_CURRENCY: [u8; 32] = *b"USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[program]
pub mod mock_oracles {
    use pyth_sdk_solana::state::Rational;

    use super::*;

    pub fn init_pyth(ctx: Context<InitPyth>) -> Result<()> {
        msg!("Mock Pyth: Init");

        let price_account_info = &ctx.accounts.price_account;
        let product_account_info = &ctx.accounts.product_account;

        // write PriceAccount
        let price_account = PriceAccount {
            magic: MAGIC,
            ver: VERSION_2,
            atype: AccountType::Price as u32,
            size: 240, // PC_PRICE_T_COMP_OFFSET from pyth_client repo
            ..PriceAccount::default()
        };

        let mut data = price_account_info.try_borrow_mut_data()?;
        data.copy_from_slice(bytemuck::bytes_of(&price_account));

        // write ProductAccount
        let attr = {
            let mut attr: Vec<u8> = Vec::new();
            let quote_currency = b"quote_currency";
            attr.push(quote_currency.len() as u8);
            attr.extend(quote_currency);
            attr.push(QUOTE_CURRENCY.len() as u8);
            attr.extend(QUOTE_CURRENCY);

            let mut buf = [0; PROD_ATTR_SIZE];
            buf[0..attr.len()].copy_from_slice(&attr);

            buf
        };

        let product_account = ProductAccount {
            magic: MAGIC,
            ver: VERSION_2,
            atype: AccountType::Product as u32,
            size: PROD_ACCT_SIZE as u32,
            px_acc: *price_account_info.key,
            attr,
        };

        let mut data = product_account_info.try_borrow_mut_data()?;
        data.copy_from_slice(bytemuck::bytes_of(&product_account));

        Ok(())
    }

    pub fn set_pyth_price(
        ctx: Context<Write>,
        price: i64,
        conf: u64,
        expo: i32,
        ema_price: i64,
        ema_conf: u64,
    ) -> Result<()> {
        msg!("Mock Pyth: Set price");
        let data = &mut ctx.accounts.target.try_borrow_mut_data()?;
        let mut price_account: &mut PriceAccount = load_mut(data).unwrap();

        price_account.agg.price = price;
        price_account.agg.conf = conf;
        price_account.expo = expo;

        price_account.ema_price = Rational {
            val: ema_price,
            // these fields don't matter
            numer: 1,
            denom: 1,
        };

        price_account.ema_conf = Rational {
            val: ema_conf as i64,
            numer: 1,
            denom: 1,
        };

        price_account.last_slot = Clock::get()?.slot;
        price_account.agg.pub_slot = Clock::get()?.slot;
        price_account.agg.status = PriceStatus::Trading;

        Ok(())
    }

    pub fn init_switchboard(ctx: Context<Write>) -> Result<()> {
        let mut data = ctx.accounts.target.try_borrow_mut_data()?;

        let discriminator = [217, 230, 65, 101, 201, 162, 27, 125];
        data[0..8].copy_from_slice(&discriminator);

        Ok(())
    }

    pub fn set_switchboard_price(ctx: Context<Write>, price: i64, expo: i32) -> Result<()> {
        msg!("Mock Switchboard: Set Switchboard price");
        let switchboard_feed = &ctx.accounts.target;
        let data = switchboard_feed.try_borrow_mut_data()?;

        let mut aggregator_account: RefMut<AggregatorAccountData> = RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data[8..std::mem::size_of::<AggregatorAccountData>() + 8])
        });

        aggregator_account.min_oracle_results = 1;
        aggregator_account.latest_confirmed_round.num_success = 1;
        aggregator_account.latest_confirmed_round.result = SwitchboardDecimal {
            mantissa: price as i128,
            scale: expo as u32,
        };
        aggregator_account.latest_confirmed_round.round_open_slot = Clock::get()?.slot;

        Ok(())
    }
}

pub fn load_mut<T: Pod>(data: &mut [u8]) -> std::result::Result<&mut T, PodCastError> {
    let size = size_of::<T>();
    Ok(from_bytes_mut(cast_slice_mut::<u8, u8>(
        try_cast_slice_mut(&mut data[0..size])?,
    )))
}

#[derive(Accounts)]
pub struct Write<'info> {
    /// CHECK: this program is just for testing
    #[account(mut)]
    pub target: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitPyth<'info> {
    /// CHECK: this program is just for testing
    #[account(mut)]
    pub price_account: AccountInfo<'info>,
    /// CHECK: this program is just for testing
    #[account(mut)]
    pub product_account: AccountInfo<'info>,
}
