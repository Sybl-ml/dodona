<template>
  <div>
    <b-container fluid>
      <b-row>
        <b-col xs="12" order="2" order-lg="1" lg="3">
          <b-row>
            <b-col class="mb-2">
              <b-form-input
                class="shadow-sm"
                v-model="search"
                placeholder="Search"
                block
              />
            </b-col>
          </b-row>
          <b-row class="text-left">
            <b-col>
              <router-link :to="{ name: 'AddProject' }">
                <b-button variant="primary" class="mb-2 shadow-sm add-new" block
                  ><b-icon-plus-circle /> Add new project
                </b-button></router-link
              >
              <router-link
                v-for="p in filtered_projects"
                :key="p._id"
                :to="{
                  name: `ProjectView`,
                  params: {
                    projectId: p._id,
                  },
                }"
              >
                <b-card
                  class="mb-2 shadow-sm"
                  no-body
                  :class="p.status.toLowerCase()"
                  style="border: none"
                >
                  <b-row
                    no-gutters
                    class="ml-2"
                    style="background-color: white"
                  >
                    <b-col>
                      <b-card-body title-tag="h5">
                        <b-card-title>{{ p.name }} </b-card-title>
                        <b-card-text>
                          <b-icon-play-fill
                            v-if="p.status == 'Unfinished'"
                            style="color: #ff643d"
                          />
                          <b-icon-play-fill
                            v-else-if="p.status == 'Ready'"
                            style="color: #6391ff"
                          />
                          <b-icon-hourglass-split
                            v-if="p.status == 'Processing'"
                            animation="fade"
                            style="color: #ffc12f"
                          />
                          <b-icon-check2-circle
                            v-else-if="p.status == 'Completed'"
                            style="color: #00bf26"
                          />
                          {{ p.status }}
                          <b-badge
                            pill
                            variant="success"
                            class="mx-1"
                            v-for="tag in p.tags"
                            v-bind:key="tag.id"
                            >{{ tag }}</b-badge
                          >
                        </b-card-text>
                      </b-card-body>
                    </b-col>
                  </b-row>
                </b-card>
              </router-link>
            </b-col>
          </b-row>
        </b-col>
        <b-col lg="9" order="1" class="mb-4">
          <router-view></router-view>
        </b-col>
      </b-row>
    </b-container>

    <particles-bg color="#cccccc" num="150" type="cobweb" :bg="true" />
  </div>
</template>

<style>
.add-new {
  height: 2.5rem;
  font-size: large;
}

.unfinished {
  background-color: #ff643d !important;
}
.ready {
  background-color: #6391ff !important;
}
.processing {
  background-color: #ffc12f !important;
}
.completed {
  background-color: #00bf26 !important;
}
</style>

<script>
import Vue from "vue";
import { ParticlesBg } from "particles-bg-vue";

export default {
  name: "Dashboard",
  data() {
    return {
      search: "",
    };
  },
  components: {
    ParticlesBg,
  },
  async created() {
    await this.$store.dispatch("getProjects");
  },
  methods: {
    async addProject(id) {},
  },
  computed: {
    filtered_projects() {
      return this.$store.getters.filteredProjects(this.search);
    },
    projects() {
      console.log(this.$store.state.projects);
      return this.$store.state.projects;
    },
  },
};
</script>
