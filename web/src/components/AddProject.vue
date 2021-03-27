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
          <h5 v-if="file && file.file">File Uploaded: {{ file.file.name }}</h5>
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
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit() {
      this.submitted = true;
      this.upload_in_progress = true;
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
        this.processFile()
      }
      
      await this.sleep(100);
      
      this.$router.replace("/dashboard/" + this.project_id);
      this.$emit("insert:project", this.project_id);
    },
    async sendFile(file, name) {
      let project_response = await this.$http.put(
        `api/projects/${this.project_id}/data`,
        {
          name: name,
          content: file,
        }
      );
    },
    async processFile() {
      if (this.file.file) {
        try {
          const file = await readUploadedFileAsText(this.file.file);
          this.sendFile(file, this.file.file.name);
        } catch (e) {
          console.warn(e.message);
        }
      }
      else if (this.file.train){
        try {
          let train = await readUploadedFileAsText(this.file.train);
          let predict = await readUploadedFileAsText(this.file.predict);
          let trainLine = train.split('\n')[0]
          let predictLine = predict.split('\n')[0]

          if (trainLine == predictLine) {
            let lines = predict.split('\n');
            lines.splice(0,1);
            let file = train + lines.join('\n');
            
            this.sendFile(file, this.file.train.name);
          }
        } catch (e) {
          console.warn(e.message);
        }
      }
    }
  },
};
</script>
