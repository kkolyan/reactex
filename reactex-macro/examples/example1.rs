use syn::{ExprClosure, parse_quote};

fn main() {

    let x: ExprClosure = parse_quote! {
            |victim_pos: &Position, health: Mut<Health>| {
                if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
                    #[update(ctx)]
                    health.health -= explosion.damage;
                }
            }
        };
    let x1: Vec<_> = x.inputs.iter().collect();
    println!();
}