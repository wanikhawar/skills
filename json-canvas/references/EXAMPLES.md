# JSON Canvas Examples

These examples follow the vault preference that Markdown note cards are raw-wikilink text nodes. File nodes are reserved for attachments.

## Spacious concept map

```json
{
  "nodes": [
    {
      "id": "01a2b3c4d5e6f789",
      "type": "group",
      "x": -80,
      "y": -100,
      "width": 1080,
      "height": 420,
      "label": "Second-law concepts",
      "color": "5"
    },
    {
      "id": "11a2b3c4d5e6f789",
      "type": "text",
      "x": 0,
      "y": 0,
      "width": 300,
      "height": 90,
      "text": "[[Clausius Inequality]]"
    },
    {
      "id": "21a2b3c4d5e6f789",
      "type": "text",
      "x": 420,
      "y": 0,
      "width": 300,
      "height": 90,
      "text": "[[Entropy Generation]]"
    },
    {
      "id": "31a2b3c4d5e6f789",
      "type": "text",
      "x": 420,
      "y": 190,
      "width": 300,
      "height": 90,
      "text": "[[Irreversibility]]"
    }
  ],
  "edges": [
    {
      "id": "41a2b3c4d5e6f789",
      "fromNode": "11a2b3c4d5e6f789",
      "fromSide": "right",
      "toNode": "21a2b3c4d5e6f789",
      "toSide": "left",
      "label": "yields"
    },
    {
      "id": "51a2b3c4d5e6f789",
      "fromNode": "21a2b3c4d5e6f789",
      "fromSide": "bottom",
      "toNode": "31a2b3c4d5e6f789",
      "toSide": "top",
      "label": "measures"
    }
  ]
}
```

## Notes with a PDF attachment

```json
{
  "nodes": [
    {
      "id": "61a2b3c4d5e6f789",
      "type": "text",
      "x": 0,
      "y": 100,
      "width": 320,
      "height": 90,
      "text": "[[Rankine Cycle]]"
    },
    {
      "id": "71a2b3c4d5e6f789",
      "type": "file",
      "x": 500,
      "y": 0,
      "width": 420,
      "height": 540,
      "file": "Sources/Thermodynamics Textbook.pdf",
      "subpath": "#page=412"
    }
  ],
  "edges": [
    {
      "id": "81a2b3c4d5e6f789",
      "fromNode": "61a2b3c4d5e6f789",
      "fromSide": "right",
      "toNode": "71a2b3c4d5e6f789",
      "toSide": "left",
      "label": "source"
    }
  ]
}
```

## Decision loop

For an ordinary process diagram, prefer Mermaid. Use this Canvas form only when the nodes need to remain spatially explorable.

```json
{
  "nodes": [
    {"id":"91a2b3c4d5e6f789","type":"text","x":200,"y":0,"width":220,"height":80,"text":"Define system"},
    {"id":"a1a2b3c4d5e6f789","type":"text","x":200,"y":180,"width":220,"height":100,"text":"Check assumptions"},
    {"id":"b1a2b3c4d5e6f789","type":"text","x":560,"y":180,"width":220,"height":80,"text":"Apply balance law"},
    {"id":"c1a2b3c4d5e6f789","type":"text","x":-160,"y":180,"width":220,"height":80,"text":"Revise model","color":"2"}
  ],
  "edges": [
    {"id":"d1a2b3c4d5e6f789","fromNode":"91a2b3c4d5e6f789","fromSide":"bottom","toNode":"a1a2b3c4d5e6f789","toSide":"top"},
    {"id":"e1a2b3c4d5e6f789","fromNode":"a1a2b3c4d5e6f789","fromSide":"right","toNode":"b1a2b3c4d5e6f789","toSide":"left","label":"valid"},
    {"id":"f1a2b3c4d5e6f789","fromNode":"a1a2b3c4d5e6f789","fromSide":"left","toNode":"c1a2b3c4d5e6f789","toSide":"right","label":"invalid"},
    {"id":"01b2c3d4e5f6a789","fromNode":"c1a2b3c4d5e6f789","fromSide":"top","toNode":"91a2b3c4d5e6f789","toSide":"left"}
  ]
}
```
