trait Thing {}

struct B;
impl Thing for B {}

struct HelloWorld<'a, T: 'a + Thing> {
	some_field: &'a T
}

fn main () {

	let hello = HelloWorld {
		some_field: &B
	};

}
