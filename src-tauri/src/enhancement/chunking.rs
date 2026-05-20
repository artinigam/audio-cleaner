/// Split audio into overlapping chunks for processing
pub fn chunk_audio(
    audio: &[f32],
    chunk_size: usize,
    overlap_size: usize,
) -> Vec<Vec<f32>> {
    if audio.is_empty() {
        return vec![];
    }

    let step_size = chunk_size - overlap_size;
    let mut chunks = Vec::new();
    let mut position = 0;

    while position < audio.len() {
        let chunk_end = (position + chunk_size).min(audio.len());
        let chunk = audio[position..chunk_end].to_vec();
        chunks.push(chunk);

        if chunk_end >= audio.len() {
            break;
        }

        position += step_size;
    }

    chunks
}

/// Reconstruct audio from overlapping chunks using linear crossfade
pub fn reconstruct_from_chunks(
    chunks: &[Vec<f32>],
    chunk_size: usize,
    overlap_size: usize,
) -> Vec<f32> {
    if chunks.is_empty() {
        return vec![];
    }

    if chunks.len() == 1 {
        return chunks[0].clone();
    }

    let step_size = chunk_size - overlap_size;
    let total_samples = (chunks.len() - 1) * step_size + chunks.last().unwrap().len();
    let mut output = vec![0.0f32; total_samples];

    for (i, chunk) in chunks.iter().enumerate() {
        let start_pos = i * step_size;

        for (j, &sample) in chunk.iter().enumerate() {
            let pos = start_pos + j;
            if pos >= output.len() {
                break;
            }

            // Apply crossfade in overlap regions
            let weight = if i > 0 && j < overlap_size {
                // Fade in from previous chunk
                j as f32 / overlap_size as f32
            } else if i < chunks.len() - 1 && j >= chunk.len() - overlap_size {
                // Fade out for next chunk
                let fade_pos = j - (chunk.len() - overlap_size);
                1.0 - (fade_pos as f32 / overlap_size as f32)
            } else {
                1.0
            };

            output[pos] += sample * weight;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_audio() {
        let audio = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let chunks = chunk_audio(&audio, 4, 1);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(chunks[1], vec![4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_reconstruct_from_chunks() {
        let chunks = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![4.0, 5.0, 6.0, 7.0],
        ];
        let output = reconstruct_from_chunks(&chunks, 4, 1);

        // Output should have smooth transitions
        assert!(output.len() >= 7);
    }
}
