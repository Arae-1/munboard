"use strict";

const misc = require("../util/misc.js");
const CustomFieldList = require("../models/custom_field_list.js");
const AutoCompleteControl = require("./auto_complete_control.js");

class CustomFieldAutoCompleteControl extends AutoCompleteControl {
    constructor(input, options) {
        options.getMatches = (text) => {
            const term = misc.escapeSearchTerm(text);

            return new Promise((resolve, reject) => {
                CustomFieldList.get(["id", "name", "description", "color"]).then(
                    (response) => {
                        const matches = response.results
                            .filter((cf) =>
                                cf.name.toLowerCase().includes(text.toLowerCase())
                            )
                            .slice(0, this._options.maxResults)
                            .map((cf) => {
                                const regex = new RegExp(
                                    "(" + misc.escapeSearchTerm(text) + ")",
                                    "gi"
                                );
                                const caption = misc
                                    .escapeHtml(cf.name)
                                    .replace(regex, "<b>$1</b>");
                                return {
                                    caption: caption,
                                    value: cf,
                                };
                            });
                        resolve(matches);
                    },
                    reject
                );
            });
        };

        super(input, options);
    }
}

module.exports = CustomFieldAutoCompleteControl;
