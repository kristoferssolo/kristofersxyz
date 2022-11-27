from django.urls import path
from . import views

urlpatterns = [
    path("", views.karbs, name="karbs"),
    path("instructions", views.instructions, name="instructions"),
]
