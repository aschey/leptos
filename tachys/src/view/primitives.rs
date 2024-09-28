use super::{Mountable, Render};
use crate::renderer::Renderer;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8,
        NonZeroIsize, NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64,
        NonZeroU8, NonZeroUsize,
    },
};

// any changes here should also be made in src/reactive_graph/guards.rs
macro_rules! render_primitive {
  ($($child_type:ty),* $(,)?) => {
    $(
		paste::paste! {
			pub struct [<$child_type:camel State>]<R>(<R as Renderer>::Text, $child_type) where R: Renderer;

			impl<R: Renderer> Mountable<R> for [<$child_type:camel State>]<R> {
					fn unmount(&mut self) {
						self.0.unmount()
					}

					fn mount(
						&mut self,
						parent: &<R as Renderer>::Element,
						marker: Option<&<R as Renderer>::Node>,
					) {
						R::insert_node(parent, self.0.as_ref(), marker);
					}

					fn insert_before_this(&self,
						child: &mut dyn Mountable<R>,
					) -> bool {
						self.0.insert_before_this(child)
					}
			}

			impl<R: Renderer> Render<R> for $child_type {
				type State = [<$child_type:camel State>]<R>;


				fn build(self) -> Self::State {
					let node = R::create_text_node(&self.to_string());
					[<$child_type:camel State>](node, self)
				}

				fn rebuild(self, state: &mut Self::State) {
					let [<$child_type:camel State>](node, this) = state;
					if &self != this {
						R::set_text(node, &self.to_string());
						*this = self;
					}
				}
			}
		}
    )*
  };
}

render_primitive![
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    char,
    bool,
    IpAddr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
    Ipv4Addr,
    Ipv6Addr,
    NonZeroI8,
    NonZeroU8,
    NonZeroI16,
    NonZeroU16,
    NonZeroI32,
    NonZeroU32,
    NonZeroI64,
    NonZeroU64,
    NonZeroI128,
    NonZeroU128,
    NonZeroIsize,
    NonZeroUsize,
];
