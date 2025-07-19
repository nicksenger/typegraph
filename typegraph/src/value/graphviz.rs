use super::{EdgeKind, EdgeKindWithIxs, NodeKind, ValueGraph};

pub trait Graphviz {
    fn render() -> String;
}
impl<T> Graphviz for T
where
    T: ValueGraph<NodeKind>,
{
    fn render() -> String {
        use petgraph::dot::{Config, Dot};
        use petgraph::graph::{EdgeReference, NodeIndex};
        use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};
        use std::collections::HashMap;
        let graph = <Self as ValueGraph<NodeKind>>::value();

        #[derive(Default)]
        struct Cluster(HashMap<String, Box<Cluster>>, String, usize);
        impl Cluster {
            fn depth(&self) -> usize {
                self.2
            }

            fn register(&mut self, cluster: &[&str]) {
                if let Some(c) = cluster.first() {
                    let entry = self.0.entry(c.to_string()).or_default();
                    entry.1 = c.to_string();
                    entry.2 = self.2 + 1;
                    if cluster.len() > 1 {
                        entry.register(&cluster[1..]);
                    }
                }
            }

            fn render(
                &self,
                graph: petgraph::Graph<NodeKind, EdgeKindWithIxs<NodeIndex>>,
                args: &HashMap<NodeIndex, HashMap<u32, &&str>>,
                properties: &HashMap<NodeIndex, HashMap<u32, &&str>>,
            ) -> String {
                let binding = |_, er: EdgeReference<'_, EdgeKindWithIxs<NodeIndex>>| {
                    let meta = match er.weight().kind {
                        EdgeKind::Argument => args
                            .get(&er.target())
                            .and_then(|x| x.get(&er.weight().from))
                            .unwrap_or(&&""),
                        EdgeKind::Property => properties
                            .get(&er.source())
                            .and_then(|x| x.get(&er.weight().to))
                            .unwrap_or(&&""),
                        _ => &&"",
                    };
                    let label = er.weight().label(meta);
                    let attr = format!(
                        "label = \"{}\" arrowhead = \"{}\" weight = {} penwidth = {} color = \"{}bf\" fontcolor = \"{}b3\"",
                        label,
                        er.weight().kind.arrowhead(),
                        er.weight().kind.weight(),
                        er.weight().kind.penwidth(),
                        er.weight().kind.color(),
                        er.weight().kind.color(),
                    );
                    attr
                };

                let s = {
                    let dot = Dot::with_attr_getters(
                        &graph,
                        &[Config::NodeNoLabel, Config::EdgeNoLabel],
                        &binding,
                        &|_, (_, x)| {
                            format!(
                                "label = \"{}\" shape = \"{}\" style=\"filled\" fillcolor = \"{}26\" fontcolor = \"{}\" color = \"{}\" subgraph=[{}]",
                                x.label(),
                                x.shape(),
                                x.color(),
                                x.color(),
                                x.color(),
                                x.cluster().join(",")
                            )
                        },
                    );
                    format!("{:?}", &dot)
                };

                let edges = s.lines().filter(|l| l.contains(" -> ")).collect::<Vec<_>>();
                let nodes = s
                    .lines()
                    .filter(|l| l.contains("subgraph=["))
                    .collect::<Vec<_>>();

                let (main_nodes, sub_nodes): (Vec<_>, Vec<_>) = nodes
                    .iter()
                    .map(|s| s.to_string())
                    .partition(|s| s.contains("subgraph=[]"));

                let paths = sub_nodes
                    .iter()
                    .cloned()
                    .filter_map(|s| s.split("subgraph=[").nth(1).map(|s| s.to_string()))
                    .filter_map(|s| s.split("]").next().map(|s| s.to_string()))
                    .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
                    .collect::<Vec<_>>();

                let sub_nodes = sub_nodes
                    .into_iter()
                    .zip(paths)
                    .map(|(s, p)| {
                        let spath = p.join(",");
                        let m = format!("{spath}]");
                        (s.replace("subgraph=[", "").replace(&m, ""), p)
                    })
                    .collect();

                let subgraphs = self
                    .render_subgraph(sub_nodes)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| format!("    {l}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                let main_nodes = main_nodes
                    .into_iter()
                    .map(|s| s.to_string().replace("subgraph=[]", ""))
                    .map(|s| format!("    {s}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "digraph {{fontcolor = \"#a9b1d6\"\n    bgcolor = \"#1a1b26\"    \n    fontname = \"NewCenturySchlbk-Bold\"\n    subgraph main {{\n{}\n    }}\n{}\n{}\n}}",
                    main_nodes,
                    subgraphs,
                    edges.join("\n")
                )
            }

            fn render_subgraph(&self, nodes: Vec<(String, Vec<String>)>) -> String {
                let gs = self
                    .0
                    .iter()
                    .map(|(cluster_name, cluster)| {
                        let (nodes, sub): (Vec<_>, Vec<_>) = nodes
                            .iter()
                            .filter(|(_, v)| v.first() == Some(cluster_name))
                            .map(|(s, p)| {
                                (s.clone(), p.iter().skip(1).cloned().collect::<Vec<_>>())
                            })
                            .partition(|(_, v)| v.is_empty());
                        let subgraphs = cluster.render_subgraph(sub);

                        let capitalized_cname = cluster_name
                            .replace("_", " ")
                            .split(" ")
                            .map(|s| s.chars().enumerate().map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c }).collect::<String>())
                            .collect::<Vec<_>>()
                            .join(" ");
                        format!("subgraph cluster_{} {{\n    label = \"{}\"\n    style = \"filled\"\n    color = \"#a9b1d6\"\n    fillcolor = \"{}\"\n{}\n{}\n}}",
                            cluster_name,
                            capitalized_cname,
                            if cluster.depth() % 2 == 0 {
                                "#1a1b26"
                            } else {
                                "#24283b"
                            },
                            subgraphs.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n"),
                            nodes
                                .into_iter()
                                .map(|(s, _)| format!("    {}", s.trim()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    })
                    .collect::<Vec<_>>();

                gs.join("\n")
            }
        }

        let mut cluster = Cluster::default();
        let mut properties = HashMap::new();
        let mut args = HashMap::new();
        graph.node_references().for_each(|r| {
            cluster.register(r.weight().cluster());
            match r.weight() {
                NodeKind::Variant(_, _, v) | NodeKind::Struct(_, _, v) => {
                    let props = v.iter().map(|(s, n)| (*n, s)).collect::<HashMap<_, _>>();
                    properties.insert(r.id(), props);
                }
                NodeKind::Function(_, _, v) | NodeKind::AsyncFunction(_, _, v) => {
                    let ags = v.iter().map(|(s, n)| (*n, s)).collect::<HashMap<_, _>>();
                    args.insert(r.id(), ags);
                }
                _ => {}
            }
        });

        cluster.render(graph, &args, &properties)
    }
}
