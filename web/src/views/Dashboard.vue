<template>
  <div>
    <b-container fluid>
      <b-row>
        <b-col xs="12" order-xs="2" lg="3">
          <b-row>
            <b-col class="mb-2">
              <b-form-input class="shadow-sm" v-model="search" placeholder="Search" block />
            </b-col>
          </b-row>
          <b-row class="text-left">
            <b-col>
              <router-link :to="{ name: 'AddProject' }">
                <b-button variant="primary" class="mb-2 shadow-sm add-new" block
                  ><b-row
                    ><b-col class="text-left">Add new project</b-col
                    ><b-col class="ml-auto text-right">
                      <b-icon-plus-circle /></b-col></b-row></b-button
              ></router-link>
              <router-link
                v-for="p in filtered_projects"
                :key="p.id"
                :to="{
                  name: `ProjectView`,
                  params: {
                    projectId: p.id,
                  },
                }"
              >
                <b-card class="mb-2 shadow-sm" no-body :class="p.status.toLowerCase()" style="border: none">
                  <b-row
                    no-gutters
                    class="ml-2"
                    style="background-color: white"
                  >
                    <b-col>
                      <b-card-body :title="p.name" title-tag="h5">
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
                            style="color: #FFC12F"
                          />
                          <b-icon-check2-circle
                            v-else-if="p.status == 'Completed'"
                            style="color: #00bf26"
                          />
                          {{ p.status }}
                        </b-card-text>
                      </b-card-body>
                    </b-col>
                  </b-row>
                </b-card>
              </router-link>
            </b-col>
          </b-row>
        </b-col>
        <b-col lg="9">
          <router-view
            @update:description="updateDescription"
            @update:name="updateName"
            @delete:project="deleteProject"
            @update:add="addProject"
          ></router-view>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<style>
.add-new {
  height: 40px;
  font-size: large;
}

.unfinished {
  background-color: #ff643d !important;
}
.ready {
  background-color: #6391ff !important;
}
.processing {
  background-color: #FFC12F !important;
}
.completed {
  background-color: #00bf26 !important;
}
</style>

<script>
import axios from "axios";
import Vue from "vue";

export default {
  name: "Dashboard",
  data() {
    return {
      projects: [],
      search: "",
    };
  },
  async mounted() {
    let user_id = $cookies.get("token");

    let response = await axios.get(
      `http://localhost:3001/api/projects/u/${user_id}`
    );

    this.projects = response.data.map((x) => {
      let y = {
        ...x,
        id: x._id.$oid,
        user_id: x.user_id.$oid,
        date_created: x.date_created.$date,
      };
      delete y._id;
      return y;
    });
  },
  methods: {
    updateName(newName, id) {
      console.log(newName, id);
      for (var i in this.projects) {
        if (this.projects[i].id == id) {
          Vue.set(this.projects[i], "name", newName);
          break;
        }
      }
    },
    updateDescription(newDescription, id) {
      console.log(newDescription, id);
      for (var i in this.projects) {
        if (this.projects[i].id == id) {
          Vue.set(this.projects[i], "description", newDescription);
          break;
        }
      }
    },
    async addProject(id) {
      let project_response = await axios.get(
        `http://localhost:3001/api/projects/p/${id}`
      );

      let x = project_response.data.project;
      let y = {
        ...x,
        id: x._id.$oid,
        user_id: x.user_id.$oid,
        date_created: x.date_created.$date,
      };
      delete y._id;

      this.projects.push(y)
    },
    deleteProject(id) {
      let index = 0;
      for (var i in this.projects) {
        if (this.projects[i].id == id) {
          index = i;
          break;
        }
      }

      this.projects.splice(index, 1);
    },
  },
  computed: {
    filtered_projects: function () {
      return this.projects.filter((x) => {
        if (x["name"].includes(this.search)) {
          return x;
        }
      });
    },
  },
};
</script>
