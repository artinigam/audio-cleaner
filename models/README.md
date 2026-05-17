# DeepFilterNet ONNX Model

This directory should contain the DeepFilterNet ONNX model file for audio enhancement.

## Obtaining the Model

### Option 1: Download Pre-converted ONNX Model (Recommended)

If the DeepFilterNet team provides pre-converted ONNX models, download from:
- https://github.com/Rikorose/DeepFilterNet/releases

Look for files named:
- `deepfilternet.onnx`
- `deepfilternet2.onnx` (or similar)
- `DeepFilterNet3_onnx.tar.gz` (extract to get .onnx file)

### Option 2: Convert PyTorch Model to ONNX

If you need to convert the model yourself:

1. **Install DeepFilterNet Python package:**
   ```bash
   pip install deepfilternet
   ```

2. **Export to ONNX format:**
   ```python
   import torch
   from df.enhance import init_df
   
   # Load the model
   model, df_state, _ = init_df()
   model.eval()
   
   # Create dummy input (adjust dimensions based on model)
   dummy_input = torch.randn(1, 1, 48000)  # [batch, channels, samples]
   
   # Export to ONNX
   torch.onnx.export(
       model,
       dummy_input,
       "deepfilternet.onnx",
       export_params=True,
       opset_version=15,
       do_constant_folding=True,
       input_names=['input'],
       output_names=['output'],
       dynamic_axes={
           'input': {2: 'num_samples'},
           'output': {2: 'num_samples'}
       }
   )
   ```

3. **Verify the exported model:**
   ```python
   import onnx
   model = onnx.load("deepfilternet.onnx")
   onnx.checker.check_model(model)
   print("Model is valid!")
   ```

## Model Requirements

- **Input Format:** Float32 tensor, shape `[batch, channels, samples]`
  - Batch size: 1
  - Channels: 1 (mono)
  - Samples: Variable length (48kHz sample rate)
  
- **Output Format:** Float32 tensor, same shape as input
  - Enhanced audio samples in range [-1.0, 1.0]

- **Sample Rate:** 48000 Hz (48kHz)

## File Placement

Place the downloaded or converted `deepfilternet.onnx` file in this directory:
```
audio-cleaner/
├── models/
│   └── deepfilternet.onnx  ← Place model file here
```

## Model Metadata

Expected model properties:
- **Input node name:** `input`
- **Output node name:** `output`
- **Frame size:** 480 samples (10ms @ 48kHz) or 96 samples
- **Model size:** ~5-50 MB depending on version

## Alternative: Download from Hugging Face

Some pre-trained models may be available on Hugging Face:
```bash
# Install huggingface-cli
pip install huggingface-hub

# Download model (if available)
huggingface-cli download Rikorose/DeepFilterNet --include "*.onnx" --local-dir ./
```

## Troubleshooting

### Model Not Found Error
If you see "DeepFilterNet ONNX model not found", ensure:
1. The file is named exactly `deepfilternet.onnx`
2. It's placed in the `models/` directory
3. The file path is accessible to the Tauri application

### ONNX Runtime Errors
If inference fails:
1. Check ONNX opset version compatibility (use opset 13-15)
2. Verify input/output node names match the code
3. Ensure model expects correct input shape

### Performance Issues
For faster inference:
1. Use GPU acceleration (CUDA) if available
2. Reduce chunk size in `deepfilternet.rs`
3. Consider quantized model versions (INT8)

## Model Licenses

DeepFilterNet models are typically licensed under MIT or Apache 2.0.
Check the original repository for specific licensing terms:
https://github.com/Rikorose/DeepFilterNet
