"use strict";

const assert = require("assert");
const templates = require("../js/templates.js");

function makePool(id, name, parent) {
    return {
        id: id,
        names: [name],
        parent: parent,
        searchableName: name,
        category: "default",
        postCount: 0,
        creationTime: "2026-01-01T00:00:00Z"
    };
}

function getRow(html, type, name) {
    const rows = html.match(new RegExp(
        '<tr class="' + type + '[^>]*>[\\s\\S]*?<\\/tr>', "g")) || [];
    const row = rows.find((item) => item.includes(">" + name + "</a>"));
    assert(row, "Expected a " + type + " row for " + name);
    return row;
}

function getGuideClasses(html, name) {
    const row = getRow(html, "child", name);
    return [...row.matchAll(/<span class="tree-guide ([^"]+)">/g)]
        .map((match) => match[1]);
}

const rootG = makePool(1, "g", null);
const childG = makePool(2, "g-child", {id: 1});
const rootQ = makePool(3, "q", null);
const childQ = makePool(4, "q-child", {id: 3});
const grandchildQ = makePool(5, "q-grandchild", {id: 4});
const html = templates["pools-page"]({
    hierarchy: true,
    parameters: {},
    response: {results: [rootG, childG, rootQ, childQ, grandchildQ]},
    formatClientLink: (route, parameters) => "/" + route + "/" + (parameters.query || parameters),
    makeCssName: () => "pool-default",
    getPrettyName: (name) => name,
    makeRelativeTime: () => "now"
});

assert.deepStrictEqual(getGuideClasses(html, "g-child"), ["connector last-child"]);
assert(!getRow(html, "root", "q").includes("tree-guides"));
assert.deepStrictEqual(getGuideClasses(html, "q-child"), ["connector last-child"]);
assert.deepStrictEqual(getGuideClasses(html, "q-grandchild"), [
    "ancestor ",
    "connector last-child"
]);
