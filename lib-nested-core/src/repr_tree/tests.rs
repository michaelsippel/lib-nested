
use {
    r3vi::{
        buffer::singleton::{
            SingletonBuffer
        },
        view::port::UpdateTask
    },
    crate::{
        repr_tree::{Context, ReprTreeExt, ReprTree, ReprLeaf}
    },
    std::sync::{Arc, RwLock}
};

#[test]
fn char_view() {
    let ctx = Arc::new(RwLock::new(Context::new()));
    crate::editors::digit::init_ctx( ctx.clone() );

    let mut rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );
    rt_digit.insert_leaf(
        Context::parse(&ctx, "Char"),
        ReprLeaf::from_singleton_buffer( SingletonBuffer::new('5') )
    );

    //<><><><>
    let mut digit_char_buffer = rt_digit
        .descend( Context::parse(&ctx, "Char") ).unwrap()
        .singleton_buffer::<char>();

    assert_eq!( digit_char_buffer.get(), '5' );
    //<><><><>

    let digit_char_view = rt_digit
        .descend(Context::parse(&ctx, "Char")).unwrap()
        .view_char();

    assert_eq!( digit_char_view.get_view().unwrap().get(), '5' );


    //<><><><>
    // `Char-view` is correctly coupled to `char-buffer`
    digit_char_buffer.set('2');
    assert_eq!( digit_char_view.get_view().unwrap().get(), '2' );
}

#[test]
fn digit_projection_char_to_u8() {
    let ctx = Arc::new(RwLock::new(Context::new()));
    crate::editors::digit::init_ctx( ctx.clone() );

    let mut rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );

    rt_digit.insert_leaf(
        Context::parse(&ctx, "Char"),
        ReprLeaf::from_singleton_buffer( SingletonBuffer::new('5') )
    );

    //<><><><>
    // add another representation
 
    ctx.read().unwrap().morphisms.apply_morphism(
        rt_digit.clone(),
        &Context::parse(&ctx, "<Digit 16>~Char"),
        &Context::parse(&ctx, "<Digit 16>~ℤ_256~machine::UInt8")
    );

    let digit_u8_view = rt_digit
        .descend(Context::parse(&ctx, "ℤ_256~machine::UInt8")).unwrap()
        .view_u8();

    assert_eq!( digit_u8_view.get_view().unwrap().get(), 5 as u8 );


    // projection behaves accordingly , when buffer is changed

    let mut digit_char_buffer = rt_digit
        .descend( Context::parse(&ctx, "Char") ).unwrap()
        .singleton_buffer::<char>();

    digit_char_buffer.set('2');
    assert_eq!( digit_u8_view.get_view().unwrap().get(), 2 as u8 );
}

#[test]
fn digit_projection_u8_to_char() {
    let ctx = Arc::new(RwLock::new(Context::new()));
    crate::editors::digit::init_ctx( ctx.clone() );

    let mut rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );

    rt_digit.insert_leaf(
        Context::parse(&ctx, "ℤ_256~machine::UInt8"),
        ReprLeaf::from_singleton_buffer( SingletonBuffer::new(5 as u8) )
    );

    //<><><><>
    // add another representation
 
    ctx.read().unwrap().morphisms.apply_morphism(
        rt_digit.clone(),
        &Context::parse(&ctx, "<Digit 16>~ℤ_256~machine::UInt8"),
        &Context::parse(&ctx, "<Digit 16>~Char")
    );

    let digit_u8_view = rt_digit
        .descend(Context::parse(&ctx, "Char")).unwrap()
        .view_char();

    assert_eq!( digit_u8_view.get_view().unwrap().get(), '5' );
}


#[test]
fn char_buffered_projection() {
    let ctx = Arc::new(RwLock::new(Context::new()));
    crate::editors::digit::init_ctx( ctx.clone() );

    let mut rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );

    rt_digit.insert_leaf(
        Context::parse(&ctx, "ℤ_256~machine::UInt8"),
        ReprLeaf::from_singleton_buffer( SingletonBuffer::new(8 as u8) )
    );

    let mut digit_u8_buffer = rt_digit
        .descend(Context::parse(&ctx, "ℤ_256~machine::UInt8")).unwrap()
        .singleton_buffer::<u8>();

    assert_eq!( digit_u8_buffer.get(), 8 );

    rt_digit.insert_leaf(
        Context::parse(&ctx, "Char"),
        ReprLeaf::from_singleton_buffer( SingletonBuffer::new('5') )
    );

    let digit_char_buf = rt_digit
        .descend(Context::parse(&ctx, "Char")).unwrap()
        .singleton_buffer::<char>();
    let digit_char_view = rt_digit
        .descend(Context::parse(&ctx, "Char")).unwrap()
        .view_char();

    // before setting up the morphism, char-view remains as initialized
    assert_eq!( digit_char_buf.get(), '5' );
    assert_eq!( digit_char_view.get_view().unwrap().get(), '5' );

    // now we attach the char-repr to the u8-repr
    ctx.read().unwrap().morphisms.apply_morphism(
        rt_digit.clone(),
        &Context::parse(&ctx, "<Digit 16>~ℤ_256~machine::UInt8"),
        &Context::parse(&ctx, "<Digit 16>~Char")
    );

    // char buffer and view should now follow the u8-buffer
    assert_eq!( digit_char_view.get_view().unwrap().get(), '8' );
    assert_eq!( digit_char_buf.get(), '8' );

    // now u8-buffer changes, and char-buffer should change accordingly
    digit_u8_buffer.set(3);
    assert_eq!( digit_u8_buffer.get(), 3 );

    // char buffer should follow
    digit_char_view.0.update();
    assert_eq!( digit_char_buf.get(), '3' );
    assert_eq!( digit_char_view.get_view().unwrap().get(), '3' ); 
}

