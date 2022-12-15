import { ShapeInfo, Intersection } from "kld-intersections";

const element_ids = [
  "picture",
  "check_tree",
  "check_boundaries",
  "check_arrowheads",
  "check_shortcuts",
  "check_all",
  "check_bounding_box",
  "winding_number",
  "check_winding_numbers",
];
let elements = {};
for (const el of element_ids) {
  elements[el] = document.getElementById(el);
}

const params = new URLSearchParams(document.location.search);
const filename = params.get("svg");
const svgResponse = await fetch(import.meta.env.BASE_URL + filename);

elements.picture.innerHTML = await svgResponse.text();

const svg = document.getElementsByTagName("svg")[0];
svg.innerHTML += '<g id="winding" style="pointer-events: none;"></g>';

const segments = document.getElementsByClassName("segment");
const nodes = document.getElementsByClassName("treenode");
let current_node = null;

function set_all_opacity(opacity) {
  for (const segment of segments) {
    segment.setAttribute("opacity", opacity);
  }
  for (const node of nodes) {
    node.setAttribute("opacity", opacity);
  }
}

for (const node of nodes) {
  node.addEventListener("mouseenter", () => {
    set_all_opacity(0.1);
    hide_shortcuts();
    if (document.querySelector("#check_shortcuts").checked) {
      for (const el of node.querySelectorAll(".shortcut")) {
        el.style.visibility = "visible";
      }
    }
    current_node = node;
    node.setAttribute("opacity", 1);
    const segments = node.getAttribute("segments");
    if (segments === "") {
      return;
    }
    for (const segment of segments.split(" ")) {
      document.getElementById(segment).setAttribute("opacity", 1);
    }
  });
}

function hide_shortcuts() {
  for (const el of document.querySelectorAll(".shortcut")) {
    el.style.visibility = "hidden";
  }
}

function first_point(shape) {
  return shape.args[0].args[0];
}

function checkIntersections(winding, line, elements) {
  let winding_number = 0;
  for (const el of elements) {
    const path_shape = ShapeInfo.path(el.getAttribute("d"));
    const intersections = Intersection.intersect(path_shape, line);
    for (const intersection of intersections.points) {
      const p = first_point(path_shape);
      winding_number += p.y > intersection.y ? 1 : -1;
      winding.innerHTML += `<circle cx="${intersection.x}" cy="${intersection.y}" fill="red" r="2" />`;
    }
  }
  return winding_number;
}

const winding = document.getElementById("winding");

svg.addEventListener("mouseleave", () => {
  set_all_opacity(1);
  hide_shortcuts();
  current_node = null;
  elements.winding_number.innerHTML = "";
  winding.innerHTML = "";
});

svg.addEventListener("mousemove", (e) => {
  const p = svg.createSVGPoint();
  p.x = e.clientX;
  p.y = e.clientY;
  const transformed = p.matrixTransform(svg.getScreenCTM().inverse());

  winding.innerHTML = `
    <circle cx="${transformed.x}" cy="${transformed.y}" r="2" fill="red" />
    <line x1="${transformed.x}" y1="${transformed.y}" x2="200" y2="${transformed.y}" stroke="red" />
  `;

  let winding_number =
    current_node && elements.check_winding_numbers.checked
      ? parseInt(current_node.querySelector(".winding_number").innerHTML)
      : 0;

  const line = ShapeInfo.line(
    [transformed.x, transformed.y],
    [200, transformed.y]
  );
  if (current_node) {
    const segments = current_node.getAttribute("segments");
    if (segments !== "") {
      for (const segment of segments.split(" ")) {
        winding_number += checkIntersections(
          winding,
          line,
          document.querySelectorAll(`#${segment} path`)
        );
      }
    }
    if (elements.check_shortcuts.checked && elements.check_tree.checked) {
      winding_number += checkIntersections(
        winding,
        line,
        current_node.querySelectorAll(".shortcut")
      );
    }
  } else {
    winding_number += checkIntersections(
      winding,
      line,
      document.querySelectorAll(".segment path")
    );
  }

  elements.winding_number.innerHTML = winding_number.toString();
  elements.winding_number.style.top = `calc(${e.clientY}px - 0.7em)`;
});

function set_class_by_check(
  check_el,
  target_query,
  classname,
  remove,
  initial
) {
  check_el.checked = initial;
  const f = () => {
    const cond = check_el.checked ? !remove : remove;
    for (const el of document.querySelectorAll(target_query)) {
      el.classList.toggle(classname, cond);
    }
  };
  check_el.addEventListener("change", f);
  f();
}

set_class_by_check(elements.check_tree, ".treenode", "invisible", true, false);
set_class_by_check(
  elements.check_boundaries,
  ".segmentBoundary",
  "invisible",
  true,
  false
);
set_class_by_check(
  elements.check_arrowheads,
  ".firstHalf",
  "arrowhead",
  false,
  false
);
set_class_by_check(
  elements.check_bounding_box,
  ".segment_bounding_box",
  "invisible",
  true,
  false
);
set_class_by_check(
  elements.check_winding_numbers,
  ".winding_number",
  "invisible",
  true,
  false
);

elements.check_all.addEventListener("change", () => {
  const checked = elements.check_all.checked;
  const event = new Event("change");
  for (const el of document.querySelectorAll('input[type="checkbox"]')) {
    if (el === check_all) {
      continue;
    }
    el.checked = checked;
    el.dispatchEvent(event);
  }
});

const original_paths = document.querySelectorAll(".original");
for (const path of original_paths) {
  path.style.stroke = "none";
  path.style.fill = "none";
}

const fill_rules = [
  document.getElementById("fill_none"),
  document.getElementById("fill_nonzero"),
  document.getElementById("fill_evenodd"),
]

for (const fill_rule of fill_rules) {
  fill_rule.addEventListener("change", () => {
    if (fill_rule.checked) {
      for (const path of original_paths) {
        if (fill_rule.value === "none") {
          path.style.fill = "none";
        } else {
          if (elements.check_tree.checked) {
            path.style.fill = "#eee";
          } else {
            path.style.fill = "black";
          }
          path.style["fill-rule"] = fill_rule.value;
        }
      }
    }
  })
}

elements.check_tree.addEventListener("change", () => {
  if (!document.getElementById("fill_none").checked) {
    for (const path of original_paths) {
      path.style.fill = elements.check_tree.checked ? "#eee" : "black";
    }
  }
})