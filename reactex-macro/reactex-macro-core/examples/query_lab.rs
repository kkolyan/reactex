use quote::quote;
use lab_helper::print;
use reactex_macro_core::lab_helper;

fn main() {
    let attr = quote!(ctx);
    let item = quote!{
        |victim_pos: &Position, health: Mut<Health>| {
            if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
                #[modify(ctx)]
                || health.health -= 6;

                #[modify(ctx)]
                || health.health -= explosion.damage;
            }
        };
    };
    let result = reactex_macro_core::query::query_attr(attr, item);
    println!("{}", print(result));
}