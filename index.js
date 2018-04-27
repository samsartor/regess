import "./main.scss"
import regess from "./Cargo.toml"
import * as d3 from "d3"
import "d3-graphviz"

import $ from "jquery"
import "popper.js"
import "bootstrap"

let gv = d3.select("#graph").graphviz();
gv.transition(function () {
    return d3.transition("main")
        .ease(d3.easeLinear)
        .duration(300);
});

let err_out = $("#error-alert-outside");
let err = $("#error-alert");

class Alerter {
    static show(text, mode) {
        err.attr("class", "alert alert-" + mode);
        err.text(text);
        err_out.collapse("show");
    }

    static dismiss() {
        err_out.collapse("hide");
    }
}

regess.set_alert(Alerter);
regess.set_display(gv);
