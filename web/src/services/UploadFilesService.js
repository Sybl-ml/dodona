class UploadFilesService {
  upload(file, onUploadProgress, projectId) {
    let formData = new FormData();

    formData.append("file", file);

    return this.$http.put(`api/projects/${projectId}/data`, formData, {
      headers: {
        "Content-Type": "multipart/form-data"
      },
      onUploadProgress
    });
  }

  getFiles() {
    return this.$http.get("/files");
  }
}

export default new UploadFilesService();