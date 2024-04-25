# <a name="imports">World imports</a>


 - Imports:
    - interface `cosmonic-labs:cloud-metadata/types@0.1.0`

## <a name="cosmonic_labs:cloud_metadata_types_0.1.0"></a>Import interface cosmonic-labs:cloud-metadata/types@0.1.0


----

### Types

#### <a name="region"></a>`record region`


##### Record Fields

- <a name="region.name"></a>`name`: `string`
- <a name="region.code"></a>`code`: option<`string`>
- <a name="region.city"></a>`city`: option<`string`>
#### <a name="metadata"></a>`record metadata`


##### Record Fields

- <a name="metadata.private_ip"></a>`private-ip`: `string`
- <a name="metadata.public_ip"></a>`public-ip`: option<`string`>
- <a name="metadata.region"></a>`region`: [`region`](#region)
- <a name="metadata.id"></a>`id`: `string`
