use super::rrdtool::Target;

use log::trace;

/// Wrapper for graph arguments to share interface between plugins
#[derive(Debug)]
pub struct GraphArguments {
    /// Local or Remote
    pub target: Target,
    /// Arguments
    /// First dimension splits it between files,
    /// Second dimension holds the arguments
    pub args: Vec<Vec<String>>,
}

impl GraphArguments {
    pub fn new(target: Target) -> GraphArguments {
        GraphArguments {
            target: target,
            args: Vec::new(),
        }
    }

    /// Create new output file for following commands
    pub fn new_graph(&mut self) {
        self.args.push(Vec::new())
    }

    /// Add new graph argument
    ///
    /// # Arguments
    ///
    /// * `legend_name` - name to be shown on graph legend
    /// * `color` - color of line, e.g. #ffaabb
    /// * `thickness` - line thickness
    /// * `path` - full path to rrd file
    ///
    pub fn push(&mut self, legend_name: &str, color: &str, thickness: u32, path: &str) {
        let legend_first_word = legend_name.split_whitespace().next().unwrap();

        let def = self.build_graph_def(legend_first_word, path);
        let line = self.build_graph_line(legend_first_word, legend_name, color, thickness);

        if self.args.last_mut() == None {
            self.args.push(Vec::new());
        }

        trace!(
            "Pushed new GraphArguments[{}][{}]:\n{:?}\n{:?}",
            self.args.len(),
            self.args.last().unwrap().len(),
            def,
            line
        );

        self.args.last_mut().unwrap().push(def);
        self.args.last_mut().unwrap().push(line);
    }

    fn build_graph_def(&mut self, unique_name: &str, path: &str) -> String {
        String::from("DEF:")
            + unique_name
            + "="
            + match self.target {
                Target::Local => "",
                Target::Remote => "\"",
            }
            + path
            + match self.target {
                Target::Local => "",
                Target::Remote => "\"",
            }
            + ":value:AVERAGE"
    }

    fn build_graph_line(
        &mut self,
        unique_name: &str,
        legend_name: &str,
        color: &str,
        thickness: u32,
    ) -> String {
        String::from("LINE")
            + &thickness.to_string()
            + ":"
            + unique_name
            + color
            + ":\""
            + legend_name
            + "\""
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn build_graph_line() -> Result<()> {
        let mut graph_arguments_local = super::GraphArguments::new(Target::Local);
        let mut graph_arguments_remote = super::GraphArguments::new(Target::Remote);

        let res_local =
            graph_arguments_local.build_graph_line("unique_name", "legend name", "#abcdef", 3);

        let res_remote = graph_arguments_remote.build_graph_line(
            "other_unique_name",
            "remote legend name",
            "#fedcba",
            5,
        );

        assert_eq!("LINE3:unique_name#abcdef:\"legend name\"", res_local);
        assert_eq!(
            "LINE5:other_unique_name#fedcba:\"remote legend name\"",
            res_remote
        );

        Ok(())
    }

    #[test]
    fn build_graph_def() -> Result<()> {
        let mut graph_arguments_local = super::GraphArguments::new(Target::Local);
        let mut graph_arguments_remote = super::GraphArguments::new(Target::Remote);

        let res_local =
            graph_arguments_local.build_graph_def("local_unique_name", "/some/local/path.rrd");
        let res_remote =
            graph_arguments_remote.build_graph_def("remote_unique_name", "/some/remote/path.rrd");

        assert_eq!(
            "DEF:local_unique_name=/some/local/path.rrd:value:AVERAGE",
            res_local
        );

        assert_eq!(
            "DEF:remote_unique_name=\"/some/remote/path.rrd\":value:AVERAGE",
            res_remote
        );

        Ok(())
    }

    #[test]
    fn graph_arguments_push() -> Result<()> {
        let mut graph_arguments_local = super::GraphArguments::new(Target::Local);
        let mut graph_arguments_remote = super::GraphArguments::new(Target::Remote);

        graph_arguments_local.push("unique legend name", "#ffaabb", 3, "/some/local/path.rrd");
        graph_arguments_remote.push("remote legend name", "#bbaaff", 5, "/some/remote/path.rrd");

        assert_eq!(1, graph_arguments_local.args.len());
        assert_eq!(2, graph_arguments_local.args[0].len());

        assert_eq!(1, graph_arguments_remote.args.len());
        assert_eq!(2, graph_arguments_remote.args[0].len());

        Ok(())
    }
}
