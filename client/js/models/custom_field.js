"use strict";

const api = require("../api.js");
const uri = require("../util/uri.js");
const events = require("../events.js");

class CustomField extends events.EventTarget {
    constructor() {
        super();
        this._orig = {};

        this._updateFromResponse({});
    }

    get id() {
        return this._id;
    }

    get name() {
        return this._name;
    }

    get description() {
        return this._description;
    }

    get color() {
        return this._color;
    }

    get creationTime() {
        return this._creationTime;
    }

    get lastEditTime() {
        return this._lastEditTime;
    }

    set name(value) {
        this._name = value;
    }

    set description(value) {
        this._description = value;
    }

    set color(value) {
        this._color = value;
    }

    static fromResponse(response) {
        const ret = new CustomField();
        ret._updateFromResponse(response);
        return ret;
    }

    static get(id) {
        return api.get(uri.formatApiLink("custom-field", id)).then((response) => {
            return Promise.resolve(CustomField.fromResponse(response));
        });
    }

    save() {
        const detail = { version: this._version };

        if (this._name !== this._orig._name) {
            detail.name = this._name;
        }
        if (this._description !== this._orig._description) {
            detail.description = this._description;
        }
        if (this._color !== this._orig._color) {
            detail.color = this._color;
        }

        let promise = this._id
            ? api.put(uri.formatApiLink("custom-field", this._id), detail)
            : api.post(uri.formatApiLink("custom-fields"), detail);
        return promise.then((response) => {
            this._updateFromResponse(response);
            this.dispatchEvent(
                new CustomEvent("change", {
                    detail: {
                        customField: this,
                    },
                })
            );
            return Promise.resolve();
        });
    }

    delete() {
        return api
            .delete(uri.formatApiLink("custom-field", this._id), {
                version: this._version,
            })
            .then((response) => {
                this.dispatchEvent(
                    new CustomEvent("delete", {
                        detail: {
                            customField: this,
                        },
                    })
                );
                return Promise.resolve();
            });
    }

    _updateFromResponse(response) {
        const map = {
            _id: response.id,
            _version: response.version,
            _name: response.name,
            _description: response.description,
            _color: response.color,
            _creationTime: response.creationTime,
            _lastEditTime: response.lastEditTime,
        };

        Object.assign(this, map);
        Object.assign(this._orig, map);
    }
}

module.exports = CustomField;
