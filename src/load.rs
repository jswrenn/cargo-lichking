use std::collections::HashSet;

use cargo::core::dependency::Kind;
use cargo::core::{ Package, Workspace };
use cargo::ops;
use cargo::CargoResult;

use options::SelectedPackage;

pub fn resolve_roots<'cfg>(
        workspace: &Workspace<'cfg>,
        package: SelectedPackage) -> CargoResult<Vec<Package>> {
    Ok(match package {
        SelectedPackage::All => {
            workspace.members().cloned().collect()
        }
        SelectedPackage::Default => {
            vec![workspace.current()?.clone()]
        }
        SelectedPackage::Specific(spec) => {
            let (packages, _) = ops::resolve_ws(&workspace)?;
            let package_id = spec.query(packages.package_ids())?;
            vec![packages.get(package_id)?.clone()]
        }
    })
}

pub fn resolve_packages<'a, 'cfg, I: IntoIterator<Item=&'a Package>>(
        workspace: &Workspace<'cfg>,
        roots: I) -> CargoResult<Vec<Package>> {
    let (packages, resolve) = ops::resolve_ws(&workspace)?;

    let mut result = HashSet::new();
    let mut to_check = roots.into_iter().map(|p| p.package_id()).collect::<Vec<_>>();
    while let Some(id) = to_check.pop() {
        if let Ok(package) = packages.get(id) {
            if result.insert(package) {
                let deps = resolve.deps_not_replaced(id);
                for dep_id in deps {
                    let dep = package.dependencies().iter()
                        .find(|d| d.matches_id(dep_id))
                        .expect(&format!("Looking up a packages dependency in the package failed, failed to find '{}' in '{}'", dep_id, id));
                    if let Kind::Normal = dep.kind() {
                        let dep_id = resolve.replacement(dep_id).unwrap_or(dep_id);
                        to_check.push(dep_id);
                    }
                }
            }
        }
    }

    Ok(result.into_iter().cloned().collect())
}
