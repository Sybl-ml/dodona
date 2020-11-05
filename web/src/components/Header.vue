<template>
  <b-navbar>
    <b-navbar-brand :to="this.logoRoute">
      <icon-logo width="5em" height="3em" :show_text="true" />
    </b-navbar-brand>
    <b-navbar-nav v-if="this.loggedIn">
      <b-nav-item disabled> {{time}} </b-nav-item>
    </b-navbar-nav>
    <b-navbar-nav v-else>
      <b-nav-item>Product</b-nav-item>
      <b-nav-item>Meet the Team</b-nav-item>
      <b-nav-item>Pricing</b-nav-item>
    </b-navbar-nav>

    <b-navbar-nav class="ml-auto" v-if="this.loggedIn">
      <b-nav-item disabled>Credits: £20.20</b-nav-item>
      <b-nav-item-dropdown right>
        <template #button-content>
          <img src="https://www.w3schools.com/w3images/avatar6.png" class="img-circle">
          {{name}}
        </template>
        <b-dropdown-item disabled>{{email}}</b-dropdown-item>
        <b-dropdown-divider></b-dropdown-divider>
        <b-dropdown-item href="#">My Profile</b-dropdown-item>
        <b-dropdown-item href="#">Help</b-dropdown-item>
        <b-dropdown-divider></b-dropdown-divider>
        <b-dropdown-item @click="logout">Logout</b-dropdown-item>
        </b-nav-item-dropdown>
    </b-navbar-nav>

    <b-navbar-nav v-else class="ml-auto">
      <b-nav-form>
        <b-nav-item><router-link to="/login">Sign In</router-link></b-nav-item>
        <b-button variant="primary" to="/register">SIGN UP NOW</b-button>
      </b-nav-form>
    </b-navbar-nav>

  </b-navbar>
</template>


<style>

.img-circle {
  height: 2rem;
  border-radius: 50%;
}

</style>

<script>
import IconBase from './IconBase'
import IconLogo from './icons/IconLogo'
import axios from 'axios'

export default {
  name: "Header",
  components: {
    IconLogo
  },
  data() {
    return {
      name: "test",
      email: "",
      time: "",
      logoRoute: "/",
      loggedIn: false,
    }
  },
  methods: {
    logout: function () {
      $cookies.remove("token");
      this.$router.push('/');
    }
  },
  async mounted() {
    let user_id = $cookies.get("token");

    try {
      let user_data = await axios.get(
        `http://localhost:3001/api/users/${user_id}`
      );
      this.name = user_data.data.first_name + " " + user_data.data.last_name;
      this.email = user_data.data.email;
    } catch (err) {
      console.log(err);
    }
  },
  created() {
    let user_id = $cookies.get("token");

    if (user_id) {
        this.loggedIn = true
        this.logoRoute = "/dashboard";
    }
    else {
        this.loggedIn = false
        this.logoRoute = "/";
    }

    setInterval(() => {
      this.time = new Date();
    }, 1000)
  }
};
</script>
