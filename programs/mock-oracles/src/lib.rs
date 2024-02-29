use anchor_lang::prelude::*;
use bytemuck::Pod;
use pyth_sdk_solana::state::{
    AccountType, PriceAccount, PriceStatus, PriceType, ProductAccount, Rational, MAGIC,
    PROD_ACCT_SIZE, PROD_ATTR_SIZE, VERSION_2,
};
use std::cell::RefMut;
use std::mem::size_of;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

declare_id!("6PLWdUXJJRYeTsCHv72iwubm43E1Z1HChkyC3cQHCEtD");

const QUOTE_CURRENCY: [u8; 32] = *b"USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[program]
pub mod mock_oracles {
    use super::*;

    pub fn init_pyth(ctx: Context<InitPyth>) -> Result<()> {
        msg!("Mock Pyth: Init");

        let price_account_info = &ctx.accounts.price_account;
        let product_account_info = &ctx.accounts.product_account;

        // write PriceAccount
        let mut price_account = load_account_as_mut::<PriceAccount>(price_account_info)?;
        price_account.magic = MAGIC;
        price_account.ver = VERSION_2;
        price_account.atype = AccountType::Price as u32;
        price_account.size = size_of::<PriceAccount>() as u32;
        price_account.ptype = PriceType::Price;

        // write ProductAccount
        let attr: [u8; 464] = {
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

        let mut product_account = load_account_as_mut::<ProductAccount>(product_account_info)?;
        product_account.magic = MAGIC;
        product_account.ver = VERSION_2;
        product_account.atype = AccountType::Product as u32;
        product_account.size = PROD_ACCT_SIZE as u32;
        product_account.px_acc = *price_account_info.key;
        product_account.attr = attr;

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
        let mut price_account = load_account_as_mut::<PriceAccount>(&ctx.accounts.target)?;

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
        msg!("Mock Switchboard: Init Switchboard");
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

pub fn load_account_as_mut<'a, T: Pod>(
    account: &'a AccountInfo,
) -> std::result::Result<RefMut<'a, T>, ProgramError> {
    let data = account.try_borrow_mut_data()?;

    Ok(RefMut::map(data, |data| {
        bytemuck::from_bytes_mut(&mut data[0..size_of::<T>()])
    }))
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
