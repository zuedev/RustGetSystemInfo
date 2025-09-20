# Set the strict mode
Set-StrictMode -Version Latest

# Define the name of the image and the container
$imageName = "rust-getsysteminfo-windows-builder"
$containerName = "rust-getsysteminfo-windows-builder-container"

# Write a message to the console
Write-Host "Building the Docker image..."

# Build the Docker image
docker build -t $imageName -f Dockerfile.windows .

# Write a message to the console
Write-Host "Creating a container from the image..."

# Create a container from the image
docker create --name $containerName $imageName

# Write a message to the console
Write-Host "Copying the executable from the container..."

# Copy the executable from the container to the local filesystem
docker cp "${containerName}:/RustGetSystemInfo.exe" "./RustGetSystemInfo.exe"

# Write a message to the console
Write-Host "Removing the container..."

# Remove the container
docker rm $containerName

# Write a message to the console
Write-Host "Build complete. The executable is available at ./RustGetSystemInfo.exe"