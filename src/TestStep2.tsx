import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

interface MediaFile {
  path: string;
  format: string;
  duration_secs: number;
  video_streams: VideoStream[];
  audio_streams: AudioStream[];
}

interface VideoStream {
  index: number;
  codec: string;
  width: number;
  height: number;
  fps: number;
  bitrate?: number;
}

interface AudioStream {
  index: number;
  codec: string;
  sample_rate: number;
  channels: number;
  bitrate?: number;
}

function TestStep2() {
  const [videoPath, setVideoPath] = useState('/Users/artinigam/path/to/video.mp4');
  const [outputPath, setOutputPath] = useState('/tmp/test_audio.wav');
  const [mediaInfo, setMediaInfo] = useState<MediaFile | null>(null);
  const [status, setStatus] = useState('Ready to test');
  const [error, setError] = useState('');

  const testProbe = async () => {
    try {
      setStatus('Probing video file...');
      setError('');

      const info = await invoke<MediaFile>('probe_media_file', {
        path: videoPath
      });

      setMediaInfo(info);
      setStatus('✅ Probe successful!');
    } catch (err) {
      setError(`❌ Probe failed: ${err}`);
      setStatus('Failed');
      setMediaInfo(null);
    }
  };

  const testExtract = async () => {
    try {
      setStatus('Extracting audio (this may take a while)...');
      setError('');

      await invoke('extract_audio_from_media', {
        mediaPath: videoPath,
        outputPath: outputPath
      });

      setStatus(`✅ Audio extracted successfully to: ${outputPath}`);
    } catch (err) {
      setError(`❌ Extraction failed: ${err}`);
      setStatus('Failed');
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, -apple-system, sans-serif' }}>
      <h1>Step 2 Testing - Media Probing + Audio Extraction</h1>

      <div style={{ marginBottom: '20px', padding: '15px', background: '#f0f0f0', borderRadius: '8px' }}>
        <h3>Instructions:</h3>
        <ol>
          <li>Update the video path below to point to a real video file on your system</li>
          <li>Click "Test Probe" to read video metadata</li>
          <li>Click "Test Extract" to extract audio to WAV format (48kHz, mono, 16-bit)</li>
          <li>Verify the output with: <code>ffprobe {outputPath}</code></li>
        </ol>
      </div>

      <div style={{ marginBottom: '20px' }}>
        <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
          Video File Path:
        </label>
        <input
          type="text"
          value={videoPath}
          onChange={(e) => setVideoPath(e.target.value)}
          style={{
            width: '100%',
            padding: '8px',
            fontSize: '14px',
            border: '1px solid #ccc',
            borderRadius: '4px'
          }}
          placeholder="/path/to/your/video.mp4"
        />
      </div>

      <div style={{ marginBottom: '20px' }}>
        <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
          Output WAV Path:
        </label>
        <input
          type="text"
          value={outputPath}
          onChange={(e) => setOutputPath(e.target.value)}
          style={{
            width: '100%',
            padding: '8px',
            fontSize: '14px',
            border: '1px solid #ccc',
            borderRadius: '4px'
          }}
          placeholder="/tmp/output.wav"
        />
      </div>

      <div style={{ marginBottom: '20px', display: 'flex', gap: '10px' }}>
        <button
          onClick={testProbe}
          style={{
            padding: '10px 20px',
            fontSize: '16px',
            backgroundColor: '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          Test Probe
        </button>

        <button
          onClick={testExtract}
          style={{
            padding: '10px 20px',
            fontSize: '16px',
            backgroundColor: '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          Test Extract
        </button>
      </div>

      <div style={{
        marginBottom: '20px',
        padding: '15px',
        background: status.includes('✅') ? '#d4edda' : status.includes('❌') ? '#f8d7da' : '#fff3cd',
        border: `1px solid ${status.includes('✅') ? '#c3e6cb' : status.includes('❌') ? '#f5c6cb' : '#ffeeba'}`,
        borderRadius: '4px'
      }}>
        <strong>Status:</strong> {status}
      </div>

      {error && (
        <div style={{
          marginBottom: '20px',
          padding: '15px',
          background: '#f8d7da',
          border: '1px solid #f5c6cb',
          borderRadius: '4px',
          color: '#721c24'
        }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      {mediaInfo && (
        <div style={{ marginTop: '20px' }}>
          <h2>Media Information</h2>

          <div style={{ marginBottom: '20px' }}>
            <h3>File Details</h3>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <tbody>
                <tr style={{ borderBottom: '1px solid #ddd' }}>
                  <td style={{ padding: '8px', fontWeight: 'bold' }}>Path:</td>
                  <td style={{ padding: '8px' }}>{mediaInfo.path}</td>
                </tr>
                <tr style={{ borderBottom: '1px solid #ddd' }}>
                  <td style={{ padding: '8px', fontWeight: 'bold' }}>Format:</td>
                  <td style={{ padding: '8px' }}>{mediaInfo.format}</td>
                </tr>
                <tr style={{ borderBottom: '1px solid #ddd' }}>
                  <td style={{ padding: '8px', fontWeight: 'bold' }}>Duration:</td>
                  <td style={{ padding: '8px' }}>{mediaInfo.duration_secs.toFixed(2)} seconds</td>
                </tr>
              </tbody>
            </table>
          </div>

          {mediaInfo.video_streams.length > 0 && (
            <div style={{ marginBottom: '20px' }}>
              <h3>Video Streams ({mediaInfo.video_streams.length})</h3>
              {mediaInfo.video_streams.map((stream, idx) => (
                <div key={idx} style={{
                  marginBottom: '10px',
                  padding: '10px',
                  background: '#f8f9fa',
                  borderRadius: '4px'
                }}>
                  <strong>Stream #{stream.index}</strong>
                  <ul style={{ marginTop: '5px', marginBottom: '0' }}>
                    <li>Codec: {stream.codec}</li>
                    <li>Resolution: {stream.width}x{stream.height}</li>
                    <li>FPS: {stream.fps.toFixed(2)}</li>
                    {stream.bitrate && <li>Bitrate: {(stream.bitrate / 1000).toFixed(0)} kbps</li>}
                  </ul>
                </div>
              ))}
            </div>
          )}

          {mediaInfo.audio_streams.length > 0 && (
            <div style={{ marginBottom: '20px' }}>
              <h3>Audio Streams ({mediaInfo.audio_streams.length})</h3>
              {mediaInfo.audio_streams.map((stream, idx) => (
                <div key={idx} style={{
                  marginBottom: '10px',
                  padding: '10px',
                  background: '#f8f9fa',
                  borderRadius: '4px',
                  border: idx === 0 ? '2px solid #28a745' : 'none'
                }}>
                  <strong>Stream #{stream.index}</strong>
                  {idx === 0 && <span style={{ marginLeft: '10px', color: '#28a745' }}>← Will be used for extraction</span>}
                  <ul style={{ marginTop: '5px', marginBottom: '0' }}>
                    <li>Codec: {stream.codec}</li>
                    <li>Sample Rate: {stream.sample_rate} Hz</li>
                    <li>Channels: {stream.channels} ({stream.channels === 1 ? 'mono' : stream.channels === 2 ? 'stereo' : `${stream.channels}ch`})</li>
                    {stream.bitrate && <li>Bitrate: {(stream.bitrate / 1000).toFixed(0)} kbps</li>}
                  </ul>
                </div>
              ))}
            </div>
          )}

          <details style={{ marginTop: '20px' }}>
            <summary style={{ cursor: 'pointer', fontWeight: 'bold', padding: '10px', background: '#e9ecef', borderRadius: '4px' }}>
              Raw JSON Output
            </summary>
            <pre style={{
              marginTop: '10px',
              padding: '15px',
              background: '#f8f9fa',
              borderRadius: '4px',
              overflow: 'auto',
              fontSize: '12px'
            }}>
              {JSON.stringify(mediaInfo, null, 2)}
            </pre>
          </details>
        </div>
      )}
    </div>
  );
}

export default TestStep2;
