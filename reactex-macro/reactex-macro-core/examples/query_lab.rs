use quote::quote;
use syn::{File, parse2};
use pretty::pretty_print_expr;
use reactex_macro_core::pretty;

fn main() {
    let item = quote!{
        ctx, |victim_pos: &Position, health: Mut<Health>| {
            if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
                modify!(ctx, || health.health -= explosion.damage )
            }
        }
    };
    let result = reactex_macro_core::query::query(item);
    println!("{}", pretty_print_expr(result));
}