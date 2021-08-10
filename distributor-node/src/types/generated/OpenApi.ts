/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
  '/status': {
    /** Returns json object describing current node status. */
    'get': operations['public.status']
  }
  '/buckets': {
    /** Returns list of distributed buckets */
    'get': operations['public.buckets']
  }
  '/asset/{objectId}': {
    /** Returns a media file. */
    'get': operations['public.asset']
    /** Returns asset response headers (cache status, content type and/or length, accepted ranges etc.) */
    'head': operations['public.assetHead']
  }
}

export interface components {
  schemas: {
    'ErrorResponse': {
      'type'?: string
      'message': string
    }
    'StatusResponse': {
      'objectsInCache': number
      'storageLimit': number
      'storageUsed': number
      'uptime': number
      'downloadsInProgress'?: number
    }
    'BucketsResponse': number[]
  }
  parameters: {
    /** Data Object ID */
    'ObjectId': string
  }
  headers: {
    /** Describes cache status of an object. Hit - object is already fully fetch in distributor node's cache. Pending - object is still beeing fetched from the storage node. Miss - object is neither in cache not currently beeing fetched. Fetching from storage node may be triggered. */
    'X-Cache'?: 'hit' | 'pending' | 'miss'
    /** Describes the source of data stream. External - the request was proxied to a storage node. Local - the data is streamed from local file. */
    'X-Data-Source'?: 'external' | 'local'
  }
}

export interface operations {
  /** Returns json object describing current node status. */
  'public.status': {
    responses: {
      /** OK */
      200: {
        content: {
          'application/json': components['schemas']['StatusResponse']
        }
      }
      /** Unexpected server error */
      500: unknown
    }
  }
  /** Returns list of distributed buckets */
  'public.buckets': {
    responses: {
      /** OK */
      200: {
        content: {
          'application/json': components['schemas']['BucketsResponse']
        }
      }
      /** Unexpected server error */
      500: unknown
    }
  }
  /** Returns a media file. */
  'public.asset': {
    parameters: {
      path: {
        /** Data Object ID */
        'objectId': components['parameters']['ObjectId']
      }
    }
    responses: {
      /** Full available object data sent */
      200: {
        headers: {}
        content: {
          'image/*': string
          'audio/*': string
          'video/*': string
        }
      }
      /** Requested partial object data sent */
      206: {
        headers: {}
        content: {
          'image/*': string
          'audio/*': string
          'video/*': string
        }
      }
      /** Data object does not exist. */
      404: {
        content: {
          'application/json': components['schemas']['ErrorResponse']
        }
      }
      /** Misdirected request. Data object not supported. */
      421: {
        content: {
          'application/json': components['schemas']['ErrorResponse']
        }
      }
      /** Unexpected server error */
      500: unknown
    }
  }
  /** Returns asset response headers (cache status, content type and/or length, accepted ranges etc.) */
  'public.assetHead': {
    parameters: {
      path: {
        /** Data Object ID */
        'objectId': components['parameters']['ObjectId']
      }
    }
    responses: {
      /** Object is supported and should be send on GET request. */
      200: unknown
      /** Data object does not exist. */
      404: unknown
      /** Misdirected request. Data object not supported by the node. */
      421: unknown
      /** Unexpected server error */
      500: unknown
    }
  }
}

export interface external {}
