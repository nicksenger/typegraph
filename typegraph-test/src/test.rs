use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use raptors::Velociraptor;
use typegraph::{assert_type_eq, typegraph, Typegraph, ValueGraph, U2};
use tyranosaurs::TyranosaurusRex;

#[typegraph(generic)]
pub struct _A;
#[typegraph(generic)]
pub struct _B;
#[typegraph(generic)]
pub struct _C;
#[typegraph(generic)]
pub struct _T;
#[typegraph(generic)]
pub struct _U;
#[typegraph(generic)]
pub struct _D;
#[typegraph(generic)]
pub struct _I;
#[typegraph(generic)]
pub struct _N;
#[typegraph(generic)]
pub struct _O;
#[typegraph(generic)]
pub struct _S;

#[typegraph]
pub struct Age<A> {
    #[typegraph(force = _A)]
    contents: A,
    size: u8,
}

pub trait Carnivore {
    type Food;
    fn eat(&mut self, food: Self::Food);
}

pub mod tyranosaurs {
    use super::*;

    #[typegraph(implementations = [A, B], cluster = tyranosaurs)]
    struct TyranosaurusRex {
        pub size: u128,
        pub tooth_size: u64,
        pub digesting: Vec<TrexFood>,
        pub favorite_foods: Result<TrexFood, u8>,
        pub nose: Nostril,
        pub all_foods: TrexFood,
    }

    #[typegraph(B)]
    impl TyranosaurusRex {
        #[typegraph(generics = [_A, _B])]
        fn gogogog<A, B>(&self, name: Option<String>, age: Age<A>, birthday: B) -> Option<B> {
            todo!()
        }

        fn roar(
            &self,
            s: String,
            foo: bool,
            #[typegraph(skip)] bar: u32,
            #[typegraph(skip)] baz: Roar,
        ) -> (String, bool) {
            todo!()
        }

        fn devour<T>(&self, #[typegraph(force = String)] food: T) {}
    }

    #[typegraph]
    struct Roar(#[typegraph(skip)] pub(crate) String);

    #[typegraph(id = A, cluster = tyranosaurs)]
    impl Carnivore for TyranosaurusRex {
        type Food = TrexFood;
        fn eat(&mut self, food: TrexFood) {
            todo!()
        }
    }

    #[derive(Typegraph)]
    pub enum Nostril {
        Left { inner: LeftNostril },
        Right { inner: RightNostril },
    }

    #[derive(Typegraph)]
    pub struct LeftNostril {
        diameter: u8,
    }

    #[derive(Typegraph)]
    pub struct RightNostril {
        diameter: u8,
    }

    #[typegraph(cluster = tyranosaurs)]
    enum TrexFood {
        Brachiosaurus { a: super::long_necks::Brachiosaurus },
        //Pterodactyl { edible: Option<super::Pterodactyl> },
        Compsognathus,
        Ornithomimus,
    }
}

pub mod raptors {
    use typegraph::{typegraph, Typegraph};

    #[derive(Clone)]
    #[typegraph(implementations = [A, B], cluster = raptors)]
    pub struct Velociraptor {
        size: u64,
        #[typegraph(skip)]
        ferocity: u128,
        hunger: u128,
    }

    #[typegraph(id = A, cluster = raptors)]
    impl Velociraptor {
        pub fn into_pack_member(self) -> PackMember {
            PackMember(self)
        }

        #[typegraph(skip_ret)]
        pub fn into_pack(self) -> [Self; 4] {
            [self.clone(), self.clone(), self.clone(), self]
        }
    }

    #[derive(Clone)]
    #[typegraph(cluster = raptors)]
    pub struct PackMember(#[typegraph(skip)] Velociraptor);

    #[typegraph(id = B, cluster = raptors)]
    impl super::Carnivore for Velociraptor {
        type Food = super::Ornithomimus;
        fn eat(&mut self, food: super::Ornithomimus) {
            if food.n_feathers.is_normal() {
                self.hunger = self
                    .hunger
                    .saturating_sub((food.n_feathers % 1024.) as u128);
            } else {
                self.ferocity = self.ferocity.saturating_add(9000);
            }
        }
    }

    #[typegraph(cluster = raptors)]
    pub struct Microraptor {
        pub size: u8,
    }
}

pub mod long_necks {
    use typegraph::typegraph;

    #[typegraph(cluster = long_necks)]
    pub struct Brachiosaurus {
        pub neck_length: u128,
    }
}

#[typegraph(implementations = [A], cluster = flying_dinosaurs)]
pub struct Pterodactyl {
    wing_span: u32,
}

#[typegraph(id = A, cluster = flying_dinosaurs)]
impl Winged for Pterodactyl {
    async fn fly_away(&self, speed: f32) -> std::sync::Arc<bool> {
        todo!()
    }
}

pub trait Winged {
    async fn fly_away(&self, speed: f32) -> std::sync::Arc<bool>;
}

#[typegraph]
pub struct Ornithomimus {
    n_feathers: f64,
}

#[typegraph(implementations = [A<_T, _U>, B])]
pub enum Dinos {
    TRex { a: TyranosaurusRex },
    Velociraptor(raptors::Velociraptor),
    Microraptor(raptors::Microraptor),
    Brachiosaurus(long_necks::Brachiosaurus),
    Ornithomimus(Ornithomimus),
    Pterodactyl(Pterodactyl),
    Triceratops(Triceratops<u8, u8, u8>),
}

#[allow(clippy::type_complexity)]
#[typegraph(id = B)]
impl Dinos {
    #[typegraph(generics = [_D, _I, _N, _O, _S])]
    fn epic<D, I, N, O, S>(
        d: PhantomData<D>,
        i: Option<I>,
        n: Arc<Mutex<N>>,
        o: Result<O, S>,
    ) -> (D, (I, (N, (O, S)))) {
        todo!()
    }
}

trait Epic<T, U> {
    fn jurassic() -> T;
    fn cretaceous() -> U;
}

#[typegraph(id = A, generics = [_T, _U])]
impl<T, U> Epic<T, U> for Dinos {
    fn jurassic() -> T {
        todo!()
    }
    fn cretaceous() -> U {
        todo!()
    }
}

#[typegraph(generics = [_A, _B, _C])]
struct Triceratops<A, B, C> {
    a: Option<A>,
    b: Option<B>,
    c: Option<C>,
}

#[test]
fn graphviz() {
    use typegraph::{typegraph, Graphviz};

    #[typegraph(implementations = [A])]
    struct Foo {
        a: Bar,
        baz: Baz,
        n: u128,
    }

    #[typegraph(generic)]
    struct _M;

    #[typegraph(A)]
    impl Foo {
        fn new() -> Self {
            todo!()
        }

        #[typegraph(generics = [_M])]
        fn make_a_baz<M>(m: M) -> Baz {
            todo!()
        }
    }

    #[typegraph]
    struct Bar {
        b: String,
    }

    #[typegraph]
    struct Baz {
        b: String,
        foo: Box<Foo>,
        w: Box<Waldo>,
    }

    #[typegraph(cluster = waldo)]
    enum Waldo {
        Foo(Foo),
        Bar(Bar),
        Baz { a: Baz },
    }

    type Types = typegraph::Resolve<Dinos>;
    let s = Types::render();
    println!("{s}");
}
