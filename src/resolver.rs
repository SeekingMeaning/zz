use super::ast;
use super::parser;
use std::collections::HashMap;
use std::path::Path;


#[derive(Default)]
struct Resolver<'a> {
    modules: HashMap<String, ast::Module<'a>>,
}

pub fn resolve(namespace: &Vec<String>) -> HashMap<String, ast::Module> {
    let mut r = Resolver::default();
    let md = parser::parse(namespace.clone(), &Path::new("./src/main.zz"));
    r.modules.insert(md.namespace.join("::"), md);

    loop {
        let mut nu = Vec::new();
        for name in r.modules.keys().cloned().collect::<Vec<String>>().into_iter() {
            let mut module = r.modules.remove(&name).unwrap();
            let imports = std::mem::replace(&mut module.imports, Vec::new());
            let imports = imports.into_iter().filter_map(|mp|{
                let mut search = namespace.clone();
                search.extend(mp.namespace.iter().cloned());
                search.pop();

                if let Some(m3) = &r.modules.get(&search.join("::")) {
                    eprintln!("resolved import {} as module {}", mp.namespace.join("::"), m3.namespace.join("::"));
                    return Some(mp);
                }
                if let Some("c") = mp.namespace.first().map(|s|s.as_str()) {
                    eprintln!("resolved import {} as c system include", mp.namespace.join("::"));
                    return Some(mp);
                }

                let mut path = mp.namespace.clone();
                path.pop();
                let path     = path.join("/");

                let mut n2 = Path::new("./src").join(&path).with_extension("zz");
                if n2.exists() {
                    let mut parent = search.clone();
                    parent.pop();
                    let m = parser::parse(parent.clone(), &n2);
                    assert!(m.namespace == search , "{:?} != {:?}", m.namespace, search);
                    eprintln!("resolved import {} as module {}", mp.namespace.join("::"), m.namespace.join("::"));
                    nu.push(m);
                    Some(mp)
                } else {
                    n2 = Path::new("./src").join(&path).with_extension("h");
                    if n2.exists() {
                        eprintln!("resolved import {} as c file {:?}", mp.namespace.join("::"), n2);
                        module.includes.push(ast::Include{
                            expr: format!("{:?}", n2.canonicalize().unwrap()),
                            vis: mp.vis.clone(),
                            loc: mp.loc.clone(),
                        });
                        module.sources.extend(vec![n2.clone()]);
                    } else {
                        let e = pest::error::Error::<parser::Rule>::new_from_span(pest::error::ErrorVariant::CustomError {
                            message: format!("cannot find module"),
                        }, mp.loc.span.clone());
                        eprintln!("{} : {}", mp.loc.file, e);
                        std::process::exit(9);
                    }
                    None
                }
            });
            module.imports = imports.collect();
            r.modules.insert(name, module);
        }
        if nu.len() > 0 {
            for md in nu {
                r.modules.insert(md.namespace.join("::"), md);
            }
        } else {
            break;
        }
    }


    for name in r.modules.keys().cloned().collect::<Vec<String>>().into_iter() {
        let mut module = r.modules.remove(&name).unwrap();
        for mp in &module.imports {
            if let Some("c") = mp.namespace.first().map(|s|s.as_str()) {
                continue;
            }
            let mut search = namespace.clone();
            search.extend(mp.namespace.iter().cloned());
            search.pop();

            let m2 = &r.modules[&search.join("::")];
            module.sources.extend(m2.sources.clone());
        }
        r.modules.insert(name, module);

    }

    r.modules
}