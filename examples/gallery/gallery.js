async function loadGallery() {
  const container = document.getElementById("gallery-container");

  try {
    const response = await fetch("/list");

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const filePaths = await response.json();

    if (filePaths.length === 0) {
      container.innerHTML =
        '<div class="empty-gallery">No images uploaded yet. Upload your first image above!</div>';
      return;
    }

    const gallery = document.createElement("div");
    gallery.className = "gallery";

    filePaths.forEach((filePath) => {
      const galleryItem = createGalleryItem(filePath);
      gallery.appendChild(galleryItem);
    });

    container.innerHTML = "";
    container.appendChild(gallery);
  } catch (error) {
    console.error("Error loading gallery:", error);
    container.innerHTML = `<div class="error">Failed to load images: ${error.message}</div>`;
  }
}

function createGalleryItem(filePath) {
  const item = document.createElement("div");
  item.className = "gallery-item";

  const img = document.createElement("img");
  img.src = `/${filePath}`;
  img.alt = getFileName(filePath);
  img.loading = "lazy";

  img.onerror = function () {
    this.style.display = "none";
    const errorDiv = document.createElement("div");
    errorDiv.className = "filename";
    errorDiv.style.color = "#e74c3c";
    errorDiv.textContent = "Failed to load image";
    item.appendChild(errorDiv);
  };

  const filename = document.createElement("div");
  filename.className = "filename";
  filename.textContent = getFileName(filePath);

  item.appendChild(img);
  item.appendChild(filename);

  return item;
}

function getFileName(filePath) {
  return filePath.split("/").pop();
}

document.addEventListener("DOMContentLoaded", loadGallery);
