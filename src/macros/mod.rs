
macro_rules! on_key_pressed_impulse {
    ($keys:ident,$key_code:expr,$impulse_ref:ident,$impulse:expr) => {
        if $keys.just_pressed($key_code) {
            $impulse_ref.impulse = $impulse;
        }
    };
}
pub(super) use on_key_pressed_impulse;

macro_rules! on_key_pressed_force_set {
    ($keys:ident,$key_code:expr,$impulse_ref:ident,$force:expr) => {
        if $keys.just_pressed($key_code) {
            $impulse_ref.force = $force;
        }
    };
}
pub(super) use on_key_pressed_force_set;