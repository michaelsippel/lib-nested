use {
    r3vi::{
        view::{singleton::*, sequence::*}
    },
    crate::{
        type_system::{Context, TypeTerm, MorphismTypePattern},
        terminal::{TerminalStyle},
        editors::{list::{PTYListStyle, PTYListController}, typeterm::{State, TypeTermEditor}}
    },
    std::{sync::{Arc, RwLock}},
    cgmath::{Point2}
};

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_list_typename("Type".into()); // = Lit | Sym | App | Ladder
    ctx.add_list_typename("Type::Lit".into()); // = Num | char
    ctx.add_list_typename("Type::Lit::Num".into());  // [0-9]*
    ctx.add_list_typename("Type::Lit::Char".into()); // .
    ctx.add_list_typename("Type::Sym".into()); // = Fun | Var
    ctx.add_list_typename("Type::Sym::Fun".into());  // [a-zA-Z][a-zA-Z0-9]*
    ctx.add_list_typename("Type::Sym::Var".into());  // [a-zA-Z][a-zA-Z0-9]*
    ctx.add_list_typename("Type::App".into()); // = <T1 T2 ...>
    ctx.add_list_typename("Type::Ladder".into()); // = T1~T2~...

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type").unwrap() },
        Arc::new(move |node, _dst_type:_| {
            let new_node = TypeTermEditor::with_node( node.ctx.clone(), node.depth.get(), node.clone(), State::Any );
            Some(new_node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::Ladder").unwrap() },
        Arc::new(|mut node, _dst_type: _| {
            PTYListController::for_node( &mut node, Some('~'), None );
            PTYListStyle::for_node( &mut node, ("","~","") );
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::App").unwrap() },
        Arc::new( |mut node, _dst_type: _| {
            PTYListController::for_node( &mut node, Some(' '), Some('>') );
            PTYListStyle::for_node( &mut node, ("<"," ",">") );
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::Sym").unwrap() },
        Arc::new(|mut node, _dst_type:_| {
            PTYListController::for_node( &mut node, Some(' '), None );
            PTYListStyle::for_node( &mut node, ("","","") );
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::Sym::Fun").unwrap() },
        Arc::new(|mut node, _dst_type:_| {
            PTYListController::for_node( &mut node, Some(' '), None );
            PTYListStyle::for_node( &mut node, ("","","") );
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::Sym::Var").unwrap() },
        Arc::new(|mut node, _dst_type:_| {
            PTYListController::for_node( &mut node, Some(' '), None );
            PTYListStyle::for_node( &mut node, ("","","") );

            // display variables blue color
            if let Some(v) = node.view {
                node.view = Some(
                    v.map_item(|_i,p| p.add_style_front(TerminalStyle::fg_color((5, 120, 240)))));
            }
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("PosInt"), dst_tyid: ctx.get_typeid("Type::Lit::Num").unwrap() },
        Arc::new(|node, _dst_type:_| {
            Some(node)
        }));

    ctx.add_morphism(
        MorphismTypePattern { src_tyid: ctx.get_typeid("List"), dst_tyid: ctx.get_typeid("Type::Lit::Char").unwrap() },
        Arc::new(|mut node, _dst_type:_| {
            let mut grid = r3vi::buffer::index_hashmap::IndexBuffer::new();

            grid.insert_iter(
                vec![
                    (Point2::new(0,0), crate::terminal::make_label("'")),
                    (Point2::new(1,0), node.view.clone().unwrap()),
                    (Point2::new(2,0), crate::terminal::make_label("'")),
                ]
            );
            
            node.close_char.set(Some('\''));
            node.view = Some(
                grid.get_port()
                    .flatten()
            );

            Some(node)
        }));

    ctx.add_node_ctor("Type", Arc::new(
        |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
            Some(TypeTermEditor::new_node(ctx, depth))
        }));
}

