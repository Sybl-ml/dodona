<template>
  <b-container fluid>
    <b-card no-body style="border:none;box-shadow:none;">
      <h2>Create a New Project</h2>
    </b-card>
    <b-card no-body>
      <navigatable-tab
        :tabs="[
          { key: '1', title: '1. Details' },
          { key: '2', title: '2. Data' },
          { key: '3', title: '3. Finish' },
        ]"
      >
        <template v-slot:1>
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
          <b-form-tags
            class="mb-3"
            tag-variant="success"
            tag-pills
            remove-on-delete
            v-model="tags"
          ></b-form-tags>
        </template>

        <template v-slot:2>
          <b-card-text>
            Please upload a dataset...
          </b-card-text>
          <b-form-file
            class="mb-3"
            placeholder="Choose a file or drop it here..."
            drop-placeholder="Drop file here..."
            v-model="file"
          />
          <b-alert show variant="danger" dismissible>
            <strong>TIP:</strong> You can upload a dataset later
          </b-alert></template
        >

        <template v-slot:3>
          <b-card-text>Confirm the Details Below Before Creation</b-card-text>
          <b-table striped hover :items="reviewItems"></b-table>
          <h5>
            Selected Tags:
            <b-badge
              pill
              variant="success"
              class="mx-1"
              v-for="tag in tags"
              v-bind:key="tag.id"
              >{{ tag }}</b-badge
            >
          </h5>
          <h5 v-if="file"> File Uploaded: {{file.name}}</h5>
          <b-button
            :disabled="!name"
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
        </template>
      </navigatable-tab>
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
import NavigatableTab from "./NavigatableTab.vue";

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
      complete: true,
      submitted: false,
      tags: [],
    };
  },
  components: {
    NavigatableTab,
  },
  computed: {
    reviewItems() {
      return [{ project_name: this.name, description: this.description }];
    },
  },
  methods: {
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit() {
      this.submitted = true;
      this.upload_in_progress = true;
      let user_id = $cookies.get("token");
      try {
        let project_response = await this.$http.post(`api/projects/new`, {
          name: this.name,
          description: this.description,
          tags: this.tags,
        });

        this.project_id = project_response.data.project_id.$oid;
      } catch (err) {
        console.log(err);
      }

      if (this.file) {
        this.readFile();
      }

      await this.sleep(1000);

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
        `api/projects/${this.project_id}/data`,
        {
          name: this.file.name,
          content: e.target.result,
        }
      );
    },
  },
};
</script>
