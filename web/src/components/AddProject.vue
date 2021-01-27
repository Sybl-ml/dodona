<template>
  <b-container fluid>
    <b-form-input
      ref="title"
      placeholder="Name Your Project"
      class="add-project"
      v-model="name"
    />
    <h5>Create a new project</h5>
    <b-tabs>
      <b-tab title="New Project" active>
        <br />
        <b-form-textarea
          placeholder="Write a short description of your project"
          v-model="description"
        />
        <br />
        <b-form-checkbox v-model="show_upload">Upload data</b-form-checkbox>
        <b-form-file
          v-show="show_upload"
          placeholder="Upload a dataset"
          drop-placeholder="Drop file here..."
          v-model="file"
        />
        <br /><br />
        <b-button @click="onSubmit" variant="primary">Submit</b-button>
      </b-tab>
    </b-tabs>
  </b-container>
</template>

<style>
.add-project {
  border: none !important;
  border-radius: 0 !important;
  height: auto !important;
  font-size: 2rem !important;
  font-weight: 500 !important;
  line-height: 1.2 !important;
  padding: 0px !important;
  margin-bottom: 0.5rem !important;
}

.add-project:focus {
  box-shadow: none !important;
}
</style>

<script>
export default {
  name: "AddProject",
  data() {
    return {
      name: "",
      description: "",
      file: null,
      upload_in_progress: false,
      file_reader: null,
      project_id: "",
      show_upload: false,
    };
  },
  mounted() {
    this.$refs.title.focus();
  },
  methods: {
    async onSubmit() {
      this.upload_in_progress = true;
      let user_id = $cookies.get("token");
      try {
        let project_response = await this.$http.post(
          `http://localhost:3001/api/projects/new`,
          {
            name: this.name,
            description: this.description,
          }
        );

        this.project_id = project_response.data.project_id.$oid;
      } catch (err) {
        console.log(err);
      }

      if (this.file) {
        this.readFile();
      }
      this.$router.replace("/dashboard/"+this.project_id);
      this.$emit("insert:project", this.project_id);
    },
    readFile() {
      this.file_reader = new FileReader();
      this.file_reader.onload = this.sendFile;
      this.file_reader.readAsText(this.file);
    },
    async sendFile(e) {
      let project_response = await this.$http.put(
        `http://localhost:3001/api/projects/p/${this.project_id}/data`,
        {
          name: this.file.name,
          content: e.target.result,
        }
      );
    },
  },
};
</script>
