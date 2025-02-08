use std::marker::PhantomData;

use compact_str::CompactString;
use lru::LruCache;
use sdl2::{mixer::Chunk, AudioSubsystem};

use super::constants;

/// make chunk depend on audio system
struct ChunkEntry<'sdl> {
    chunk: Chunk,
    _phantom: PhantomData<&'sdl ()>,
}

pub struct AudioSystem<'sdl> {
    chunks: LruCache<CompactString, ChunkEntry<'sdl>>,
}

impl<'sdl> AudioSystem<'sdl> {
    pub fn new(_audio: &'sdl AudioSubsystem) -> Self {
        Self {
            chunks: LruCache::new(constants::MAX_LOADED_SOUNDS),
        }
    }

    pub fn play(&'sdl mut self, path: &str) -> Result<(), String> {
        let ret = self.chunks.try_get_or_insert_mut(path.into(), || -> Result<ChunkEntry, String> {
            let chunk = Chunk::from_file(path)?;
            // guaranteed not null. otherwise, from_file would return error and
            // not reach here
            Ok(ChunkEntry{chunk, _phantom: PhantomData })
        })?;

        // this does not expose any form of audio control, panning etc. if the
        // chunk's volume is set then this will effect previous chunks that are
        // still playing. too complicated and not worth it, at least for now

        sdl2::mixer::Channel::all().play(&ret.chunk, 0);
        Ok(())
    }
}
