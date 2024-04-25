# <a name="metadata_service">World metadata-service</a>


 - Imports:
    - interface `cosmonic-labs:cloud-metadata/types@0.1.0`
 - Exports:
    - interface `cosmonic-labs:cloud-metadata/service@0.1.0`

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
## <a name="cosmonic_labs:cloud_metadata_service_0.1.0"></a>Export interface cosmonic-labs:cloud-metadata/service@0.1.0

----

### Types

#### <a name="region"></a>`type region`
[`region`](#region)
<p>
#### <a name="metadata"></a>`type metadata`
[`metadata`](#metadata)
<p>
----

### Functions

#### <a name="get"></a>`get: func`


##### Return values

- <a name="get.0"></a> result<[`region`](#region)>

