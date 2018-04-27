import regess from "./Cargo.toml"
import * as d3 from "d3"
import "d3-graphviz"

let gv = d3.select("#graph").graphviz();
gv.transition(function () {
    return d3.transition("main")
        .ease(d3.easeLinear)
        .duration(300);
});

regess.set_display(gv);
