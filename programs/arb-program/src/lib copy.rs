use anchor_lang::prelude::*;

declare_id!("FdZ2xqz4rvcQNE5KRiHKib8pC8mYNo9reheSrFwLwspr");

#[program]
pub mod arb_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
