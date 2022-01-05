use anchor_lang::prelude::*;

declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");

#[program]
pub mod write_account {
    
    
    
    use pyth_client::{Price, PriceType};
    use quick_protobuf::serialize_into_slice;
    use switchboard_program::{AggregatorState, RoundResult, SwitchboardAccountType};
    use switchboard_program::mod_AggregatorState::Configs;
    use super::*;
    /// Write data to an account
    pub fn write(ctx: Context<Write>, offset: usize, data: Vec<u8>) -> ProgramResult {
        let account_data = &mut ctx.accounts.target.try_borrow_mut_data()?;
        account_data[offset..].copy_from_slice(&data[..]);
        Ok(())
    }

    pub fn write_pyth_price(ctx: Context<Write>, price: i64, expo:i32, slot: u64) -> ProgramResult {
        let account_data = &mut ctx.accounts.target.try_borrow_mut_data()?;
        let mut price_data:Price = unsafe {
            std::mem::zeroed()
        };
        price_data.ptype = PriceType::Price;
        price_data.valid_slot = slot;
        price_data.agg.price = price;
        price_data.expo = expo;
        account_data.copy_from_slice( unsafe { &std::mem::transmute::<Price, [u8;std::mem::size_of::<Price>()]>(price_data) });
        Ok(())
    }

    pub fn write_switchboard_price(ctx: Context<Write>, price:f64, slot: u64, board_type: u8) -> ProgramResult {
        let account_data = &mut ctx.accounts.target.try_borrow_mut_data()?;
        if board_type == 0 {
            account_data[0] = SwitchboardAccountType::TYPE_AGGREGATOR as u8;
            let mut aggregator: AggregatorState = AggregatorState::default();
            aggregator.configs = Some(
                Configs {
                    min_confirmations: Some(0),
                    ..Configs::default()
                });
            let last_round_result = RoundResult {
                round_open_slot: Some(slot),
                result: Some(price),
                num_success: Some(5),
                ..RoundResult::default()
            };
            aggregator.last_round_result = Some(last_round_result);
            serialize_into_slice(&aggregator, &mut account_data[1..]).unwrap();
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Write<'info> {
    #[account(mut)]
    pub target: Signer<'info>,
}