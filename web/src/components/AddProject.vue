<template>
  <b-container fluid>
    <b-card no-body style="border:none;box-shadow:none;">
      <h2>Create a New Project</h2>
    </b-card>
    <b-card no-body>
      <b-tabs pills card vertical v-model="tabIndex">
        <b-tab title="1. Details" active>
          <b-form-input
            ref="title"
            placeholder="Name Your Project"
            class="mb-3 add-project"
            v-model="name"
          />
          <b-form-textarea
            placeholder="Write a short description of your project"
            v-model="description"
            class="mb-3"
          />
          <b-form-tags class="mb-3" v-model="descriptions"></b-form-tags>

          <b-button
            size="sm"
            class="mb-3 float-right"
            variant="primary"
            @click="tabIndex++"
            ><strong>Next</strong></b-button
          >
        </b-tab>
        <b-tab title="2. Data"
          ><b-card-text>
            Please upload a dataset...
          </b-card-text>
          <b-form-file
            class="mb-3"
            placeholder="Upload a dataset"
            drop-placeholder="Drop file here..."
            v-model="file"
          />
          <b-alert show variant="danger" dismissible>
            <strong>TIP:</strong> You can upload a dataset later
          </b-alert>
          <b-button
            size="sm"
            class="mb-3 float-left"
            variant="primary"
            @click="tabIndex--"
            ><strong>Previous</strong></b-button
          >
          <b-button
            size="sm"
            class="mb-3 float-right"
            variant="primary"
            @click="tabIndex++"
            ><strong>Next</strong></b-button
          ></b-tab
        >
        <b-tab title="3. Processing" disabled
          ><b-card-text>Describe the data...</b-card-text>
          <b-button
            size="sm"
            class="mb-3 float-left"
            variant="primary"
            @click="tabIndex--"
            ><strong>Previous</strong></b-button
          >
          <b-button
            size="sm"
            class="mb-3 float-right"
            variant="primary"
            @click="tabIndex++"
            ><strong>Next</strong></b-button
          ></b-tab
        >
        <b-tab title="4. Configure" disabled
          ><b-card-text>How long ...</b-card-text
          ><b-button
            size="sm"
            class="mb-3 float-left"
            variant="primary"
            @click="tabIndex--"
            ><strong>Previous</strong></b-button
          >
          <b-button
            size="sm"
            class="mb-3 float-right"
            variant="primary"
            @click="tabIndex++"
            ><strong>Next</strong></b-button
          ></b-tab
        >
        <b-tab title="5. Finish"
          ><b-card-text>Confirm the Details Below Before Creation</b-card-text>
          <b-table striped hover :items="reviewItems"></b-table>
          <b-button
            size="sm"
            class="mb-3 float-left"
            variant="primary"
            @click="tabIndex--"
            ><strong>Previous</strong></b-button
          >
          <b-button
            size="sm"
            @click="onSubmit"
            variant="ready"
            class="float-right"
          >
            Create
            <b-spinner v-show="submitted" small></b-spinner>
            <b-icon-check-all
              v-show="!submitted && complete"
            ></b-icon-check-all>
          </b-button>
        </b-tab>
      </b-tabs>
    </b-card>
  </b-container>
</template>

<style>
.add-project {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.2 !important;
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
      tabIndex: 1,
      complete: true,
      submitted: false,
      descriptions: [],
      reviewItems: [
        { age: 40, first_name: "Dickerson", last_name: "Macdonald" },
        { age: 21, first_name: "Larsen", last_name: "Shaw" },
        { age: 89, first_name: "Geneva", last_name: "Wilson" },
        { age: 38, first_name: "Jami", last_name: "Carney" },
      ],
    };
  },
  mounted() {
    this.$refs.title.focus();
  },
  methods: {
    async onSubmit() {
      this.submitted = true;
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
      this.$router.replace("/dashboard/" + this.project_id);
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
