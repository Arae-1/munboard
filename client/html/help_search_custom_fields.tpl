<p>Custom fields are user-defined numeric properties that can be attached to
posts, such as <code>booster_flight</code> for the number of times a booster has flown.
The fields themselves (their names,
descriptions and colors) are defined on the Custom fields page. Individual
posts can then have any of these fields set to a number, in the Custom Fields
section of the post edit form.</p>

<p>Searching with custom fields is done in two places: posts can be searched by
the values of their custom fields, and the Custom fields page can be filtered
by the fields themselves.</p>

<p><strong>Searching posts by custom fields</strong></p>

<p>Posts can be searched by the value of a custom field, using either the
<code>cf</code> or the <code>custom-field</code> token:</p>

<table>
    <tbody>
        <tr>
            <td><code>custom-field:&lt;name&gt;=&lt;value&gt;</code></td>
            <td>having given custom field set to given value</td>
        </tr>
        <tr>
            <td><code>cf:&lt;name&gt;=&lt;value&gt;</code></td>
            <td>alias of <code>custom-field</code></td>
        </tr>
    </tbody>
</table>

<p>The name must be written exactly as the field was defined. The match is
case-sensitive and does not accept wildcards. The value is a number, and can be
written in the same forms as other numeric tokens, such as
<code>score</code>:</p>

<table>
    <tbody>
        <tr>
            <td><code>cf:booster_flight=5</code></td>
            <td>will show posts with booster flight exactly equal to 5.</td>
        </tr>
        <tr>
            <td><code>cf:booster_flight=5,7</code></td>
            <td>will show posts with booster flight equal to 5 or 7.</td>
        </tr>
        <tr>
            <td><code>cf:booster_flight=5..10</code></td>
            <td>will show posts with booster flight equal to 5, 6, 7, 8, 9 or 10.</td>
        </tr>
        <tr>
            <td><code>cf:booster_flight=5..</code></td>
            <td>will show posts with booster flight equal to or greater than 5.</td>
        </tr>
        <tr>
            <td><code>cf:booster_flight=..10</code></td>
            <td>will show posts with booster flight equal to at most 10.</td>
        </tr>
    </tbody>
</table>

<p>Negating the token will show posts that do not have a custom field with a
matching value. This includes posts that do not have the custom field set at
all. For example, <code>-cf:booster_flight=5</code> will show posts whose
booster flight is not 5, as well as posts without a booster flight custom field.</p>

<p>Custom field tokens cannot be used for sorting.</p>

<h1>Example</h1>

<p>Searching for posts with following query:</p>

<pre><code>type:video cf:booster_flight=5.. -cf:fairing_serial_number=3</code></pre>

<p>will show videos with booster flight set to 5 or more, that do not have a
fairing serial number set to 3.</p>

<p>Searching for posts with <code>cf:booster_flight</code> will show an error
message, because the value is required.</p>

<p><strong>Searching the custom fields list</strong></p>

<p>The search box on the Custom fields page filters the defined fields
themselves, not the posts.</p>

<p><strong>Anonymous tokens</strong></p>

<p>Matching given text against the name, description or color of the custom
field.</p>

<p><strong>Named tokens</strong></p>

<table>
    <tbody>
        <tr>
            <td><code>name</code></td>
            <td>having given name</td>
        </tr>
        <tr>
            <td><code>color</code></td>
            <td>having given color</td>
        </tr>
    </tbody>
</table>

<p><strong>Sort style tokens</strong></p>

<table>
    <tbody>
        <tr>
            <td><code>name</code></td>
            <td>A to Z</td>
        </tr>
        <tr>
            <td><code>creation-time</code></td>
            <td>oldest to newest</td>
        </tr>
        <tr>
            <td><code>description</code></td>
            <td>description (A to Z)</td>
        </tr>
    </tbody>
</table>

<p><strong>Special tokens</strong></p>

<p>None.</p>
