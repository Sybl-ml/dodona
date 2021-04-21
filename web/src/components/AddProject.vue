<template>
  <b-container fluid>
    <b-card no-body style="border: none; box-shadow: none">
      <h2>Create a New Project</h2>
    </b-card>
    <b-overlay :show="upload_in_progress" rounded="sm">
      <template #overlay>
        <b-row class="justify-content-center">
          <h2>Creating Project</h2>
        </b-row>
        <b-row class="justify-content-center">
          <h2>Please Do Not Refresh!</h2>
        </b-row>
        <b-row class="justify-content-center">
          <b-spinner variant="primary" style="width: 3rem; height: 3rem" />
        </b-row>
      </template>
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
            <b-card-text> Please upload a dataset... </b-card-text>
            <file-upload v-model="file" />
            <b-alert show variant="primary" dismissible>
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
            <h5 v-if="file && file.file">
              File Uploaded: {{ file.file.name }}
            </h5>
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
    </b-overlay>
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
import FileUpload from "@/components/FileUpload";

const readUploadedFileAsText = (inputFile) => {
  const temporaryFileReader = new FileReader();

  return new Promise((resolve, reject) => {
    temporaryFileReader.onerror = () => {
      temporaryFileReader.abort();
      reject(new DOMException("Problem parsing input file."));
    };

    temporaryFileReader.onload = () => {
      resolve(temporaryFileReader.result);
    };
    temporaryFileReader.readAsText(inputFile);
  });
};

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export default {
  name: "AddProject",
  data() {
    return {
      name: "",
      description: "",
      file: null,
      upload_in_progress: false,
      project_id: "",
      complete: true,
      submitted: false,
      tags: [],
      train: null,
    };
  },
  components: {
    NavigatableTab,
    FileUpload,
  },
  computed: {
    reviewItems() {
      return [{ project_name: this.name, description: this.description }];
    },
  },
  methods: {
    async onSubmit() {
      this.submitted = true;
      this.upload_in_progress = true;
      try {
        let project_response = await this.$store.dispatch("postNewProject", {
          name: this.name,
          description: this.description,
          tags: this.tags,
        });

        this.project_id = project_response.data.project_id.$oid;
        this.$store.dispatch("addProject", this.project_id);
        await sleep(500);

        if (this.file) {
          await this.processFile();
        }
        this.$router.replace(`/dashboard/${this.project_id}`);
      } catch (error) {
        console.error(error);
      }
    },
    async processFile() {
      console.log("Processing uploaded data");
      if (this.file.file) {
        try {
          console.log("Processing single file");
          let payload = {
            project_id: this.project_id,
            multipart: false,
            files: this.file.file,
          };
          this.$store.dispatch("sendFile", payload);
        } catch (e) {
          console.warn(e.message);
        }
      } else if (this.file.train) {
        // Use train endpoint and predict endpoint
        console.log("Processing 2 files");
        try {
          let payload = {
            project_id: this.project_id,
            multipart: true,
            files: {
              train: this.file.train,
              predict: this.file.predict,
            },
          };
          this.$store.dispatch("sendFile", payload);
        } catch (e) {
          console.warn(e.message);
        }
      }
    },
  },
};
</script>
